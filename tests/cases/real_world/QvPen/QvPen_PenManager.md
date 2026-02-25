<!-- ci: skip-compile -->

```csharp
using TMPro;
using UdonSharp;
using UnityEngine;
using UnityEngine.Animations;
using UnityEngine.UI;
using VRC.SDK3.Data;
using VRC.SDKBase;
using VRC.Udon.Common;
using VRC.Udon.Common.Interfaces;
using Utilities = VRC.SDKBase.Utilities;

#pragma warning disable IDE0044
#pragma warning disable IDE0090, IDE1006

namespace QvPen.UdonScript
{
    [AddComponentMenu("")]
    [DefaultExecutionOrder(20)]
    [UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
    public class QvPen_PenManager : UdonSharpBehaviour
    {
        [SerializeField]
        private QvPen_Pen pen;

        public Gradient colorGradient = new Gradient();

        public float inkWidth = 0.005f;

        // Layer 0 : Default
        // Layer 9 : Player
        public int inkMeshLayer = 0;
        public int inkColliderLayer = 9;

        public Material pcInkMaterial;
        public Material questInkMaterial;

        public LayerMask surftraceMask = ~0;

        [SerializeField]
        private GameObject respawnButton;
        [SerializeField]
        private GameObject clearButton;
        [SerializeField]
        private GameObject inUseUI;

        [SerializeField]
        private Text textInUse;
        [SerializeField]
        private TextMeshPro textInUseTMP;
        [SerializeField]
        private TextMeshProUGUI textInUseTMPU;

        [SerializeField]
        private Shader _roundedTrailShader;
        public Shader roundedTrailShader => _roundedTrailShader;

        [SerializeField]
        private bool allowCallPen = true;
        public bool AllowCallPen => allowCallPen;

        private void Start()
        {
            pen._Init(this);
        }

        public override void OnPlayerJoined(VRCPlayerApi player)
        {
            if (Networking.IsOwner(pen.gameObject) && pen.IsUser)
                SendCustomNetworkEvent(NetworkEventTarget.All, nameof(StartUsing));

            if (player.isLocal)
            {
                if (Utilities.IsValid(clearButton))
                {
                    clearButtonPositionConstraint = clearButton.GetComponent<PositionConstraint>();
                    clearButtonRotationConstraint = clearButton.GetComponent<RotationConstraint>();

                    EnableClearButtonConstraints();
                }
            }
        }

        public override void OnPlayerLeft(VRCPlayerApi player)
        {
            if (Networking.IsOwner(pen.gameObject) && !pen.IsUser)
                pen.OnDrop();
        }

        public void StartUsing()
        {
            pen.isPickedUp = true;

            if (Utilities.IsValid(respawnButton))
                respawnButton.SetActive(false);
            if (Utilities.IsValid(clearButton))
                SetClearButtonActive(false);
            if (Utilities.IsValid(inUseUI))
                inUseUI.SetActive(true);

            var owner = Networking.GetOwner(pen.gameObject);

            var text = owner != null ? owner.displayName : "Occupied";

            if (Utilities.IsValid(textInUse))
                textInUse.text = text;

            if (Utilities.IsValid(textInUseTMP))
                textInUseTMP.text = text;

            if (Utilities.IsValid(textInUseTMPU))
                textInUseTMPU.text = text;
        }

        public void EndUsing()
        {
            pen.isPickedUp = false;

            if (Utilities.IsValid(respawnButton))
                respawnButton.SetActive(true);
            if (Utilities.IsValid(clearButton))
                SetClearButtonActive(true);
            if (Utilities.IsValid(inUseUI))
                inUseUI.SetActive(false);

            if (Utilities.IsValid(textInUse))
                textInUse.text = string.Empty;

            if (Utilities.IsValid(textInUseTMP))
                textInUseTMP.text = string.Empty;

            if (Utilities.IsValid(textInUseTMPU))
                textInUseTMPU.text = string.Empty;
        }

        private PositionConstraint clearButtonPositionConstraint;
        private RotationConstraint clearButtonRotationConstraint;

        private void SetClearButtonActive(bool isActive)
        {
            if (Utilities.IsValid(clearButton))
                clearButton.SetActive(isActive);
            else
                return;

            if (!isActive)
                return;

            EnableClearButtonConstraints();
        }

        private void EnableClearButtonConstraints()
        {
            if (Utilities.IsValid(clearButtonPositionConstraint))
                clearButtonPositionConstraint.enabled = true;
            if (Utilities.IsValid(clearButtonRotationConstraint))
                clearButtonRotationConstraint.enabled = true;

            SendCustomEventDelayedSeconds(nameof(_DisableClearButtonConstraints), 2f);
        }

        public void _DisableClearButtonConstraints()
        {
            if (Utilities.IsValid(clearButtonPositionConstraint))
                clearButtonPositionConstraint.enabled = false;
            if (Utilities.IsValid(clearButtonRotationConstraint))
                clearButtonRotationConstraint.enabled = false;
        }

        #region API

        public void _SetWidth(float width)
        {
            inkWidth = width;
            pen._UpdateInkData();
        }

        public void _SetMeshLayer(int layer)
        {
            inkMeshLayer = layer;
            pen._UpdateInkData();
        }

        public void _SetColliderLayer(int layer)
        {
            inkColliderLayer = layer;
            pen._UpdateInkData();
        }

        public void _SetUsingDoubleClick(bool value) => pen._SetUseDoubleClick(value);

        public void _SetEnabledLateSync(bool value) => pen._SetEnabledLateSync(value);

        public void _SetUsingSurftraceMode(bool value) => pen._SetUseSurftraceMode(value);

        public void ResetPen()
        {
            Clear();
            Respawn();
        }

        public void Respawn()
        {
            pen._Respawn();
            SetClearButtonActive(true);
        }

        public void Clear()
        {
            _ClearSyncBuffer();
            pen._Clear();
        }

        public void UndoDraw()
        {
            if (pen.isPickedUp)
                return;

            _TakeOwnership();

            pen._UndoDraw();
        }

        public void EraseOwnInk()
        {
            if (pen.isPickedUp)
                return;

            _TakeOwnership();

            pen._EraseOwnInk();
        }

        #endregion

        #region Callback

        private readonly DataList listenerList = new DataList();

        public void Register(QvPen_PenCallbackListener listener)
        {
            if (!Utilities.IsValid(listener) || listenerList.Contains(listener))
                return;

            listenerList.Add(listener);
        }

        public void OnPenPickup()
        {
            for (int i = 0, n = listenerList.Count; i < n; i++)
            {
                if (!listenerList.TryGetValue(i, TokenType.Reference, out var listerToken))
                    continue;

                var listener = (QvPen_PenCallbackListener)listerToken.Reference;

                if (!Utilities.IsValid(listener))
                    continue;

                listener.OnPenPickup();
            }
        }

        public void OnPenDrop()
        {
            for (int i = 0, n = listenerList.Count; i < n; i++)
            {
                if (!listenerList.TryGetValue(i, TokenType.Reference, out var listerToken))
                    continue;

                var listener = (QvPen_PenCallbackListener)listerToken.Reference;

                if (!Utilities.IsValid(listener))
                    continue;

                listener.OnPenDrop();
            }
        }

        #endregion

        #region Network

        public bool _TakeOwnership()
        {
            if (Networking.IsOwner(gameObject))
            {
                _ClearSyncBuffer();
                return true;
            }
            else
            {
                Networking.SetOwner(Networking.LocalPlayer, gameObject);
                return Networking.IsOwner(gameObject);
            }
        }

        private bool _isNetworkSettled = false;
        private bool isNetworkSettled
            => _isNetworkSettled || (_isNetworkSettled = Networking.IsNetworkSettled);

        [UdonSynced]
        private Vector3[] _syncedData;
        private Vector3[] syncedData
        {
            get => _syncedData;
            set
            {
                if (!isNetworkSettled)
                    return;

                _syncedData = value;

                RequestSendPackage();

                pen._UnpackData(_syncedData, QvPen_Pen_Mode.Any);
            }
        }

        [UdonSynced]
        private int inkId;
        public int InkId => inkId;

        public void _IncrementInkId() => inkId++;

        private bool isInUseSyncBuffer = false;
        private void RequestSendPackage()
        {
            if (VRCPlayerApi.GetPlayerCount() > 1 && Networking.IsOwner(gameObject) && !isInUseSyncBuffer)
            {
                isInUseSyncBuffer = true;
                RequestSerialization();
            }
        }

        public void _SendData(Vector3[] data)
        {
            if (!isInUseSyncBuffer)
                syncedData = data;
        }

        public override void OnPreSerialization()
            => _syncedData = syncedData;

        public override void OnDeserialization()
        {
            if (Networking.IsOwner(gameObject))
                return;

            syncedData = _syncedData;
        }

        public override void OnPostSerialization(SerializationResult result)
        {
            isInUseSyncBuffer = false;

            if (result.success)
                pen._UnpackData(_syncedData, QvPen_Pen_Mode.Any);
            else
                pen._EraseAbandonedInk(_syncedData);
        }

        public void _ClearSyncBuffer()
        {
            syncedData = new Vector3[] { };
            isInUseSyncBuffer = false;
        }

        #endregion
    }
}
```

```json
{
  "byteCodeHex": "000000010000001B0000000100000011000000010000001A000000090000000100000002000000090000000800000002000000010000001B0000000100000012000000010000001C000000090000000100000002000000090000000800000002000000010000001B0000000100000003000000010000001E000000010000001D00000006000000C00000000100000003000000010000001F00000006000000C10000000100000002000000090000000800000002000000010000001B0000000100000003000000010000006C00000006000000C2000000010000006C000000010000006B00000006000000C3000000010000006B000000040000017000000001000000030000000100000003000000010000006D00000009000000010000002100000006000000C100000001000000030000000100000022000000010000006F00000006000000C4000000010000006F000000010000006E00000009000000010000006E000000010000006B00000009000000010000006B00000004000001A000000001000000230000000100000024000000010000002500000006000000C50000000100000020000000010000007000000006000000C600000001000000700000000400000270000000010000000C000000010000007100000006000000C700000001000000710000000400000270000000010000000C000000010000007200000006000000C800000001000000720000000100000026000000010000001300000006000000C9000000010000000C000000010000007300000006000000C800000001000000730000000100000027000000010000001400000006000000C9000000010000002800000005000008A40000000100000002000000090000000800000002000000010000001B0000000100000003000000010000007500000006000000C20000000100000075000000010000007400000006000000C30000000100000074000000040000034400000001000000030000000100000003000000010000007600000009000000010000002100000006000000C100000001000000030000000100000022000000010000007800000006000000C400000001000000780000000100000077000000090000000100000077000000010000007400000006000000CA0000000100000074000000040000038000000001000000030000000100000003000000010000007900000009000000010000002A00000006000000C10000000100000002000000090000000800000002000000010000001B0000000100000003000000010000002C000000010000002B00000006000000C0000000010000000B000000010000007A00000006000000C7000000010000007A00000004000003FC000000010000000B000000010000002D00000006000000CB000000010000000C000000010000007B00000006000000C7000000010000007B0000000400000448000000010000002E000000010000002D000000010000002F0000000900000005000007F0000000010000000D000000010000007C00000006000000C7000000010000007C0000000400000488000000010000000D000000010000002B00000006000000CB0000000100000003000000010000007E00000006000000C2000000010000007E000000010000007D00000006000000CC000000010000007D0000000100000030000000010000008000000006000000CD00000001000000800000000400000508000000010000007D000000010000007F00000006000000CE000000050000051C0000000100000031000000010000007F00000009000000010000000E000000010000008100000006000000C70000000100000081000000040000055C000000010000000E000000010000007F00000006000000CF000000010000000F000000010000008200000006000000C70000000100000082000000040000059C000000010000000F000000010000007F00000006000000D00000000100000010000000010000008300000006000000C7000000010000008300000004000005DC0000000100000010000000010000007F00000006000000D10000000100000002000000090000000800000002000000010000001B0000000100000003000000010000002C000000010000002D00000006000000C0000000010000000B000000010000008400000006000000C700000001000000840000000400000658000000010000000B000000010000002B00000006000000CB000000010000000C000000010000008500000006000000C7000000010000008500000004000006A40000000100000032000000010000002B000000010000002F0000000900000005000007F0000000010000000D000000010000008600000006000000C7000000010000008600000004000006E4000000010000000D000000010000002D00000006000000CB000000010000000E000000010000008700000006000000C700000001000000870000000400000734000000010000008800000006000000D2000000010000000E000000010000008800000006000000CF000000010000000F000000010000008900000006000000C700000001000000890000000400000784000000010000008A00000006000000D2000000010000000F000000010000008A00000006000000D00000000100000010000000010000008B00000006000000C7000000010000008B00000004000007D4000000010000008C00000006000000D20000000100000010000000010000008C00000006000000D10000000100000002000000090000000800000002000000010000001B000000010000000C000000010000008D00000006000000C7000000010000008D0000000400000838000000010000000C000000010000002F00000006000000CB000000050000084C0000000100000002000000090000000800000002000000010000002F000000040000086400000005000008780000000100000002000000090000000800000002000000010000003300000005000008A40000000100000002000000090000000800000002000000010000001B0000000100000013000000010000008E00000006000000C7000000010000008E00000004000008E40000000100000013000000010000002B00000006000000D30000000100000014000000010000008F00000006000000C7000000010000008F00000004000009240000000100000014000000010000002B00000006000000D4000000010000003400000001000000350000000100000036000000010000003700000006000000D50000000100000002000000090000000800000002000000010000001B0000000100000013000000010000009000000006000000C7000000010000009000000004000009A80000000100000013000000010000002D00000006000000D30000000100000014000000010000009100000006000000C7000000010000009100000004000009E80000000100000014000000010000002D00000006000000D40000000100000002000000090000000800000002000000010000001B000000010000003800000001000000050000000900000001000000030000000100000003000000010000009200000009000000010000003900000006000000C10000000100000002000000090000000800000002000000010000001B000000010000003A00000001000000060000000900000001000000030000000100000003000000010000009300000009000000010000003900000006000000C10000000100000002000000090000000800000002000000010000001B000000010000003B00000001000000070000000900000001000000030000000100000003000000010000009400000009000000010000003900000006000000C10000000100000002000000090000000800000002000000010000001B000000010000003C0000000100000095000000090000000100000003000000010000003D000000010000003C00000006000000C00000000100000003000000010000003E00000006000000C10000000100000002000000090000000800000002000000010000001B000000010000003F00000001000000960000000900000001000000030000000100000040000000010000003F00000006000000C00000000100000003000000010000004100000006000000C10000000100000002000000090000000800000002000000010000001B000000010000004200000001000000970000000900000001000000030000000100000043000000010000004200000006000000C00000000100000003000000010000004400000006000000C10000000100000002000000090000000800000002000000010000001B00000001000000450000000500000CF800000001000000460000000500000C8C0000000100000002000000090000000800000002000000010000001B00000001000000030000000100000003000000010000009800000009000000010000004700000006000000C10000000100000048000000010000002B000000010000002F0000000900000005000007F00000000100000002000000090000000800000002000000010000001B0000000100000049000000050000186400000001000000030000000100000003000000010000009900000009000000010000004A00000006000000C10000000100000002000000090000000800000002000000010000001B0000000100000003000000010000002C000000010000009A00000006000000C4000000010000009A000000010000009B00000009000000010000009B0000000400000DA80000000100000002000000090000000800000002000000010000004B000000050000127800000001000000030000000100000003000000010000009C00000009000000010000004D00000006000000C10000000100000002000000090000000800000002000000010000001B0000000100000003000000010000002C000000010000009D00000006000000C4000000010000009D000000010000009E00000009000000010000009E0000000400000E580000000100000002000000090000000800000002000000010000004E000000050000127800000001000000030000000100000003000000010000009F00000009000000010000004F00000006000000C10000000100000002000000090000000800000002000000010000001B000000010000005000000001000000A100000006000000C700000001000000A100000001000000A000000006000000CA00000001000000A00000000400000EF80000000500000F30000000010000005000000001000000A200000006000000D6000000010000001500000001000000A200000001000000A000000006000000D700000001000000A00000000400000F540000000100000002000000090000000800000002000000010000005000000001000000A300000006000000D6000000010000001500000001000000A300000006000000D80000000100000002000000090000000800000002000000010000001B000000010000005100000001000000A400000009000000010000001500000001000000A500000006000000D900000001000000A400000001000000A500000001000000A600000006000000DA00000001000000A600000004000010F0000000010000001500000001000000A4000000010000005200000001000000A700000001000000A800000006000000DB00000001000000A80000000400001044000000050000104C00000005000010C800000001000000A700000001000000AA00000006000000DC00000001000000AA00000001000000A90000000900000001000000A900000001000000AB00000006000000C700000001000000AB00000004000010A800000005000010B000000005000010C800000001000000A9000000010000005300000006000000C100000001000000A4000000010000005400000001000000A400000006000000DD0000000500000FCC0000000100000002000000090000000800000002000000010000001B000000010000005100000001000000AC00000009000000010000001500000001000000AD00000006000000D900000001000000AC00000001000000AD00000001000000AE00000006000000DA00000001000000AE000000040000125C000000010000001500000001000000AC000000010000005200000001000000AF00000001000000B000000006000000DB00000001000000B000000004000011B000000005000011B8000000050000123400000001000000AF00000001000000B200000006000000DC00000001000000B200000001000000B10000000900000001000000B100000001000000B300000006000000C700000001000000B30000000400001214000000050000121C000000050000123400000001000000B1000000010000005500000006000000C100000001000000AC000000010000005400000001000000AC00000006000000DD00000005000011380000000100000002000000090000000800000002000000010000001B000000010000005600000001000000B400000006000000C300000001000000B400000004000012E000000001000000570000000500001864000000010000002B000000010000004C000000090000000100000002000000090000000800000002000000050000133400000001000000B500000006000000DE00000001000000B5000000010000005600000006000000DF0000000100000056000000010000004C00000006000000C300000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000001B000000010000001600000001000000B60000000900000001000000B6000000040000137C00000005000013A0000000010000001600000006000000E0000000010000001600000001000000B60000000900000001000000B60000000100000058000000090000000100000002000000090000000800000002000000010000001B00000001000000170000000100000059000000090000000100000002000000090000000800000002000000010000001B000000010000005B000000050000135000000001000000580000000400001428000000050000143C0000000100000002000000090000000800000002000000010000005A000000010000001700000009000000010000005C0000000500001554000000010000001700000001000000B7000000090000000100000003000000010000005E000000010000001700000006000000C00000000100000003000000010000005F000000010000005D00000006000000C00000000100000003000000010000006000000006000000C10000000100000002000000090000000800000002000000010000001B00000001000000180000000100000061000000090000000100000002000000090000000800000002000000010000001B00000001000000180000000100000054000000010000001800000006000000DD0000000100000002000000090000000800000002000000010000001B00000001000000BA00000006000000E100000001000000BA000000010000005400000001000000B900000006000000E200000001000000B900000004000015AC000000010000005600000001000000B900000006000000C300000001000000B900000001000000B80000000900000001000000B800000004000015E8000000010000001900000001000000B800000006000000CA00000001000000B8000000040000161C000000010000002B000000010000001900000009000000010000006200000006000000E30000000100000002000000090000000800000002000000010000001B00000001000000190000000400001650000000050000167400000001000000640000000100000063000000010000005A0000000900000005000014000000000100000002000000090000000800000002000000010000001B000000010000006500000005000013D000000001000000590000000100000017000000090000000100000002000000090000000800000002000000010000001B000000010000005600000001000000BB00000006000000C300000001000000BB000000040000170C000000010000000200000009000000080000000200000001000000660000000100000017000000010000005A0000000900000005000014000000000100000002000000090000000800000002000000010000001B000000010000002D000000010000001900000009000000010000006700000001000000BC00000006000000E400000001000000BC00000004000017FC000000010000001700000001000000BD000000090000000100000003000000010000005E000000010000001700000006000000C00000000100000003000000010000005F000000010000005D00000006000000C00000000100000003000000010000006000000006000000C10000000500001848000000010000001700000001000000BE0000000900000001000000030000000100000068000000010000001700000006000000C00000000100000003000000010000006900000006000000C10000000100000002000000090000000800000002000000010000001B000000010000006A000000010000005100000001000000BF00000006000000E500000001000000BF000000010000005A000000090000000500001400000000010000002D0000000100000019000000090000000100000002000000090000000800000002",
  "byteCodeLength": 6344,
  "symbols": {
    "__intnl_SystemBoolean_13": {
      "name": "__intnl_SystemBoolean_13",
      "type": "System.Boolean",
      "address": 132
    },
    "__intnl_SystemBoolean_23": {
      "name": "__intnl_SystemBoolean_23",
      "type": "System.Boolean",
      "address": 145
    },
    "__intnl_SystemBoolean_33": {
      "name": "__intnl_SystemBoolean_33",
      "type": "System.Boolean",
      "address": 171
    },
    "__gintnl_SystemUInt32_16": {
      "name": "__gintnl_SystemUInt32_16",
      "type": "System.UInt32",
      "address": 106
    },
    "allowCallPen": {
      "name": "allowCallPen",
      "type": "System.Boolean",
      "address": 18
    },
    "__intnl_SystemString_1": {
      "name": "__intnl_SystemString_1",
      "type": "System.String",
      "address": 138
    },
    "questInkMaterial": {
      "name": "questInkMaterial",
      "type": "UnityEngine.Material",
      "address": 9
    },
    "__const_SystemType_1": {
      "name": "__const_SystemType_1",
      "type": "System.Type",
      "address": 39
    },
    "__intnl_SystemObject_0": {
      "name": "__intnl_SystemObject_0",
      "type": "System.Object",
      "address": 111
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 30
    },
    "__intnl_UnityEngineTransform_0": {
      "name": "__intnl_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 114
    },
    "__const_SystemSingle_0": {
      "name": "__const_SystemSingle_0",
      "type": "System.Single",
      "address": 54
    },
    "__1_layer__param": {
      "name": "__1_layer__param",
      "type": "System.Int32",
      "address": 59
    },
    "_isNetworkSettled": {
      "name": "_isNetworkSettled",
      "type": "System.Boolean",
      "address": 22
    },
    "__intnl_SystemBoolean_16": {
      "name": "__intnl_SystemBoolean_16",
      "type": "System.Boolean",
      "address": 135
    },
    "__intnl_SystemBoolean_26": {
      "name": "__intnl_SystemBoolean_26",
      "type": "System.Boolean",
      "address": 151
    },
    "__intnl_SystemBoolean_36": {
      "name": "__intnl_SystemBoolean_36",
      "type": "System.Boolean",
      "address": 179
    },
    "__lcl_listener_VRCUdonUdonBehaviour_1": {
      "name": "__lcl_listener_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 177
    },
    "__lcl_listener_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_listener_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 169
    },
    "__gintnl_SystemUInt32_9": {
      "name": "__gintnl_SystemUInt32_9",
      "type": "System.UInt32",
      "address": 78
    },
    "__gintnl_SystemUInt32_8": {
      "name": "__gintnl_SystemUInt32_8",
      "type": "System.UInt32",
      "address": 75
    },
    "__gintnl_SystemUInt32_5": {
      "name": "__gintnl_SystemUInt32_5",
      "type": "System.UInt32",
      "address": 70
    },
    "__gintnl_SystemUInt32_4": {
      "name": "__gintnl_SystemUInt32_4",
      "type": "System.UInt32",
      "address": 69
    },
    "__gintnl_SystemUInt32_7": {
      "name": "__gintnl_SystemUInt32_7",
      "type": "System.UInt32",
      "address": 73
    },
    "__gintnl_SystemUInt32_6": {
      "name": "__gintnl_SystemUInt32_6",
      "type": "System.UInt32",
      "address": 72
    },
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 46
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 40
    },
    "__gintnl_SystemUInt32_3": {
      "name": "__gintnl_SystemUInt32_3",
      "type": "System.UInt32",
      "address": 51
    },
    "__gintnl_SystemUInt32_2": {
      "name": "__gintnl_SystemUInt32_2",
      "type": "System.UInt32",
      "address": 50
    },
    "inkId": {
      "name": "inkId",
      "type": "System.Int32",
      "address": 24
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 43
    },
    "__const_SystemBoolean_1": {
      "name": "__const_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 45
    },
    "__0_value__param": {
      "name": "__0_value__param",
      "type": "System.Boolean",
      "address": 60
    },
    "pcInkMaterial": {
      "name": "pcInkMaterial",
      "type": "UnityEngine.Material",
      "address": 8
    },
    "respawnButton": {
      "name": "respawnButton",
      "type": "UnityEngine.GameObject",
      "address": 11
    },
    "__intnl_VRCUdonUdonBehaviour_7": {
      "name": "__intnl_VRCUdonUdonBehaviour_7",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 153
    },
    "__const_SystemString_5": {
      "name": "__const_SystemString_5",
      "type": "System.String",
      "address": 42
    },
    "__const_VRCSDK3DataTokenType_0": {
      "name": "__const_VRCSDK3DataTokenType_0",
      "type": "VRC.SDK3.Data.TokenType",
      "address": 82
    },
    "__gintnl_SystemUInt32_13": {
      "name": "__gintnl_SystemUInt32_13",
      "type": "System.UInt32",
      "address": 100
    },
    "onPostSerializationResult": {
      "name": "onPostSerializationResult",
      "type": "VRC.Udon.Common.SerializationResult",
      "address": 103
    },
    "__intnl_VRCSDK3DataDataToken_1": {
      "name": "__intnl_VRCSDK3DataDataToken_1",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 163
    },
    "inkColliderLayer": {
      "name": "inkColliderLayer",
      "type": "System.Int32",
      "address": 7
    },
    "__const_VRCUdonCommonInterfacesNetworkEventTarget_0": {
      "name": "__const_VRCUdonCommonInterfacesNetworkEventTarget_0",
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "address": 36
    },
    "pen": {
      "name": "pen",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 3
    },
    "textInUse": {
      "name": "textInUse",
      "type": "UnityEngine.UI.Text",
      "address": 14
    },
    "__const_SystemString_16": {
      "name": "__const_SystemString_16",
      "type": "System.String",
      "address": 71
    },
    "__const_SystemString_17": {
      "name": "__const_SystemString_17",
      "type": "System.String",
      "address": 74
    },
    "__const_SystemString_14": {
      "name": "__const_SystemString_14",
      "type": "System.String",
      "address": 67
    },
    "__const_SystemString_15": {
      "name": "__const_SystemString_15",
      "type": "System.String",
      "address": 68
    },
    "__const_SystemString_12": {
      "name": "__const_SystemString_12",
      "type": "System.String",
      "address": 64
    },
    "__const_SystemString_13": {
      "name": "__const_SystemString_13",
      "type": "System.String",
      "address": 65
    },
    "__const_SystemString_10": {
      "name": "__const_SystemString_10",
      "type": "System.String",
      "address": 61
    },
    "__const_SystemString_11": {
      "name": "__const_SystemString_11",
      "type": "System.String",
      "address": 62
    },
    "__const_SystemString_18": {
      "name": "__const_SystemString_18",
      "type": "System.String",
      "address": 77
    },
    "__const_SystemString_19": {
      "name": "__const_SystemString_19",
      "type": "System.String",
      "address": 79
    },
    "__intnl_VRCSDKBaseVRCPlayerApi_0": {
      "name": "__intnl_VRCSDKBaseVRCPlayerApi_0",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 181
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__intnl_VRCUdonUdonBehaviour_2": {
      "name": "__intnl_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 121
    },
    "__intnl_VRCUdonUdonBehaviour_9": {
      "name": "__intnl_VRCUdonUdonBehaviour_9",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 159
    },
    "__lcl_text_SystemString_0": {
      "name": "__lcl_text_SystemString_0",
      "type": "System.String",
      "address": 127
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 27
    },
    "__intnl_SystemBoolean_11": {
      "name": "__intnl_SystemBoolean_11",
      "type": "System.Boolean",
      "address": 130
    },
    "__intnl_SystemBoolean_21": {
      "name": "__intnl_SystemBoolean_21",
      "type": "System.Boolean",
      "address": 143
    },
    "__intnl_SystemBoolean_31": {
      "name": "__intnl_SystemBoolean_31",
      "type": "System.Boolean",
      "address": 166
    },
    "__intnl_SystemBoolean_41": {
      "name": "__intnl_SystemBoolean_41",
      "type": "System.Boolean",
      "address": 187
    },
    "__gintnl_SystemUInt32_14": {
      "name": "__gintnl_SystemUInt32_14",
      "type": "System.UInt32",
      "address": 101
    },
    "__0_data__param": {
      "name": "__0_data__param",
      "type": "UnityEngine.Vector3[]",
      "address": 99
    },
    "__lcl_listerToken_VRCSDK3DataDataToken_1": {
      "name": "__lcl_listerToken_VRCSDK3DataDataToken_1",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 175
    },
    "__0_get_InkId__ret": {
      "name": "__0_get_InkId__ret",
      "type": "System.Int32",
      "address": 97
    },
    "__intnl_SystemObject_2": {
      "name": "__intnl_SystemObject_2",
      "type": "System.Object",
      "address": 154
    },
    "clearButtonRotationConstraint": {
      "name": "clearButtonRotationConstraint",
      "type": "UnityEngine.Animations.RotationConstraint",
      "address": 20
    },
    "__intnl_VRCUdonUdonBehaviour_1": {
      "name": "__intnl_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 118
    },
    "inkMeshLayer": {
      "name": "inkMeshLayer",
      "type": "System.Int32",
      "address": 6
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 33
    },
    "__intnl_SystemBoolean_19": {
      "name": "__intnl_SystemBoolean_19",
      "type": "System.Boolean",
      "address": 141
    },
    "__intnl_SystemBoolean_29": {
      "name": "__intnl_SystemBoolean_29",
      "type": "System.Boolean",
      "address": 160
    },
    "__intnl_SystemBoolean_39": {
      "name": "__intnl_SystemBoolean_39",
      "type": "System.Boolean",
      "address": 184
    },
    "__intnl_UnityEngineGameObject_2": {
      "name": "__intnl_UnityEngineGameObject_2",
      "type": "UnityEngine.GameObject",
      "address": 126
    },
    "__intnl_UnityEngineGameObject_1": {
      "name": "__intnl_UnityEngineGameObject_1",
      "type": "UnityEngine.GameObject",
      "address": 117
    },
    "__intnl_UnityEngineGameObject_0": {
      "name": "__intnl_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 108
    },
    "__0_layer__param": {
      "name": "__0_layer__param",
      "type": "System.Int32",
      "address": 58
    },
    "__intnl_SystemBoolean_14": {
      "name": "__intnl_SystemBoolean_14",
      "type": "System.Boolean",
      "address": 133
    },
    "__intnl_SystemBoolean_24": {
      "name": "__intnl_SystemBoolean_24",
      "type": "System.Boolean",
      "address": 149
    },
    "__intnl_SystemBoolean_34": {
      "name": "__intnl_SystemBoolean_34",
      "type": "System.Boolean",
      "address": 174
    },
    "__const_SystemObject_0": {
      "name": "__const_SystemObject_0",
      "type": "System.Object",
      "address": 48
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 136
    },
    "__const_SystemType_0": {
      "name": "__const_SystemType_0",
      "type": "System.Type",
      "address": 38
    },
    "__intnl_SystemObject_1": {
      "name": "__intnl_SystemObject_1",
      "type": "System.Object",
      "address": 120
    },
    "__const_SystemString_7": {
      "name": "__const_SystemString_7",
      "type": "System.String",
      "address": 49
    },
    "__gintnl_SystemUInt32_11": {
      "name": "__gintnl_SystemUInt32_11",
      "type": "System.UInt32",
      "address": 91
    },
    "__3_value__param": {
      "name": "__3_value__param",
      "type": "UnityEngine.Vector3[]",
      "address": 90
    },
    "__intnl_SystemBoolean_17": {
      "name": "__intnl_SystemBoolean_17",
      "type": "System.Boolean",
      "address": 137
    },
    "__intnl_SystemBoolean_27": {
      "name": "__intnl_SystemBoolean_27",
      "type": "System.Boolean",
      "address": 155
    },
    "__intnl_SystemBoolean_37": {
      "name": "__intnl_SystemBoolean_37",
      "type": "System.Boolean",
      "address": 180
    },
    "__const_VRCUdonCommonEnumsEventTiming_0": {
      "name": "__const_VRCUdonCommonEnumsEventTiming_0",
      "type": "VRC.Udon.Common.Enums.EventTiming",
      "address": 55
    },
    "textInUseTMPU": {
      "name": "textInUseTMPU",
      "type": "TMPro.TextMeshProUGUI",
      "address": 16
    },
    "clearButton": {
      "name": "clearButton",
      "type": "UnityEngine.GameObject",
      "address": 12
    },
    "onPlayerJoinedPlayer": {
      "name": "onPlayerJoinedPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 32
    },
    "__intnl_VRCUdonUdonBehaviour_4": {
      "name": "__intnl_VRCUdonUdonBehaviour_4",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 147
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "__const_SystemString_4": {
      "name": "__const_SystemString_4",
      "type": "System.String",
      "address": 37
    },
    "__gintnl_SystemUInt32_12": {
      "name": "__gintnl_SystemUInt32_12",
      "type": "System.UInt32",
      "address": 92
    },
    "__lcl_i_SystemInt32_1": {
      "name": "__lcl_i_SystemInt32_1",
      "type": "System.Int32",
      "address": 172
    },
    "__lcl_i_SystemInt32_0": {
      "name": "__lcl_i_SystemInt32_0",
      "type": "System.Int32",
      "address": 164
    },
    "textInUseTMP": {
      "name": "textInUseTMP",
      "type": "TMPro.TextMeshPro",
      "address": 15
    },
    "isInUseSyncBuffer": {
      "name": "isInUseSyncBuffer",
      "type": "System.Boolean",
      "address": 25
    },
    "__0_get_roundedTrailShader__ret": {
      "name": "__0_get_roundedTrailShader__ret",
      "type": "UnityEngine.Shader",
      "address": 26
    },
    "__intnl_SystemObject_4": {
      "name": "__intnl_SystemObject_4",
      "type": "System.Object",
      "address": 170
    },
    "__0_listener__param": {
      "name": "__0_listener__param",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 80
    },
    "__intnl_VRCUdonUdonBehaviour_3": {
      "name": "__intnl_VRCUdonUdonBehaviour_3",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 146
    },
    "__const_SystemString_9": {
      "name": "__const_SystemString_9",
      "type": "System.String",
      "address": 57
    },
    "__intnl_SystemBoolean_12": {
      "name": "__intnl_SystemBoolean_12",
      "type": "System.Boolean",
      "address": 131
    },
    "__intnl_SystemBoolean_22": {
      "name": "__intnl_SystemBoolean_22",
      "type": "System.Boolean",
      "address": 144
    },
    "__intnl_SystemBoolean_32": {
      "name": "__intnl_SystemBoolean_32",
      "type": "System.Boolean",
      "address": 168
    },
    "__intnl_SystemBoolean_42": {
      "name": "__intnl_SystemBoolean_42",
      "type": "System.Boolean",
      "address": 188
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "clearButtonPositionConstraint": {
      "name": "clearButtonPositionConstraint",
      "type": "UnityEngine.Animations.PositionConstraint",
      "address": 19
    },
    "__lcl_listerToken_VRCSDK3DataDataToken_0": {
      "name": "__lcl_listerToken_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 167
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 186
    },
    "__intnl_SystemString_2": {
      "name": "__intnl_SystemString_2",
      "type": "System.String",
      "address": 140
    },
    "onPlayerLeftPlayer": {
      "name": "onPlayerLeftPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 41
    },
    "colorGradient": {
      "name": "colorGradient",
      "type": "UnityEngine.Gradient",
      "address": 4
    },
    "__intnl_SystemObject_3": {
      "name": "__intnl_SystemObject_3",
      "type": "System.Object",
      "address": 157
    },
    "__0_get_syncedData__ret": {
      "name": "__0_get_syncedData__ret",
      "type": "UnityEngine.Vector3[]",
      "address": 89
    },
    "__lcl_owner_VRCSDKBaseVRCPlayerApi_0": {
      "name": "__lcl_owner_VRCSDKBaseVRCPlayerApi_0",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 125
    },
    "inUseUI": {
      "name": "inUseUI",
      "type": "UnityEngine.GameObject",
      "address": 13
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 31
    },
    "__intnl_UnityEngineTransform_1": {
      "name": "__intnl_UnityEngineTransform_1",
      "type": "UnityEngine.Transform",
      "address": 115
    },
    "__0__TakeOwnership__ret": {
      "name": "__0__TakeOwnership__ret",
      "type": "System.Boolean",
      "address": 76
    },
    "__intnl_SystemBoolean_15": {
      "name": "__intnl_SystemBoolean_15",
      "type": "System.Boolean",
      "address": 134
    },
    "__intnl_SystemBoolean_25": {
      "name": "__intnl_SystemBoolean_25",
      "type": "System.Boolean",
      "address": 150
    },
    "__intnl_SystemBoolean_35": {
      "name": "__intnl_SystemBoolean_35",
      "type": "System.Boolean",
      "address": 176
    },
    "__2_value__param": {
      "name": "__2_value__param",
      "type": "System.Boolean",
      "address": 66
    },
    "__0_get_AllowCallPen__ret": {
      "name": "__0_get_AllowCallPen__ret",
      "type": "System.Boolean",
      "address": 28
    },
    "__0_isActive__param": {
      "name": "__0_isActive__param",
      "type": "System.Boolean",
      "address": 47
    },
    "__intnl_VRCUdonUdonBehaviour_6": {
      "name": "__intnl_VRCUdonUdonBehaviour_6",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 152
    },
    "_syncedData": {
      "name": "_syncedData",
      "type": "UnityEngine.Vector3[]",
      "address": 23
    },
    "__lcl_n_SystemInt32_0": {
      "name": "__lcl_n_SystemInt32_0",
      "type": "System.Int32",
      "address": 165
    },
    "__lcl_n_SystemInt32_1": {
      "name": "__lcl_n_SystemInt32_1",
      "type": "System.Int32",
      "address": 173
    },
    "__const_SystemString_6": {
      "name": "__const_SystemString_6",
      "type": "System.String",
      "address": 44
    },
    "__0_width__param": {
      "name": "__0_width__param",
      "type": "System.Single",
      "address": 56
    },
    "__gintnl_SystemUInt32_10": {
      "name": "__gintnl_SystemUInt32_10",
      "type": "System.UInt32",
      "address": 87
    },
    "__intnl_VRCSDK3DataDataToken_0": {
      "name": "__intnl_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 162
    },
    "__0_get_isNetworkSettled__ret": {
      "name": "__0_get_isNetworkSettled__ret",
      "type": "System.Boolean",
      "address": 88
    },
    "__this_UnityEngineGameObject_0": {
      "name": "__this_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 86
    },
    "__1_value__param": {
      "name": "__1_value__param",
      "type": "System.Boolean",
      "address": 63
    },
    "__const_SystemString_26": {
      "name": "__const_SystemString_26",
      "type": "System.String",
      "address": 105
    },
    "__const_SystemString_24": {
      "name": "__const_SystemString_24",
      "type": "System.String",
      "address": 96
    },
    "__const_SystemString_25": {
      "name": "__const_SystemString_25",
      "type": "System.String",
      "address": 104
    },
    "__const_SystemString_22": {
      "name": "__const_SystemString_22",
      "type": "System.String",
      "address": 94
    },
    "__const_SystemString_23": {
      "name": "__const_SystemString_23",
      "type": "System.String",
      "address": 95
    },
    "__const_SystemString_20": {
      "name": "__const_SystemString_20",
      "type": "System.String",
      "address": 83
    },
    "__const_SystemString_21": {
      "name": "__const_SystemString_21",
      "type": "System.String",
      "address": 85
    },
    "__intnl_VRCUdonUdonBehaviour_5": {
      "name": "__intnl_VRCUdonUdonBehaviour_5",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 148
    },
    "_roundedTrailShader": {
      "name": "_roundedTrailShader",
      "type": "UnityEngine.Shader",
      "address": 17
    },
    "__intnl_VRCUdonUdonBehaviour_8": {
      "name": "__intnl_VRCUdonUdonBehaviour_8",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 156
    },
    "__intnl_SystemBoolean_10": {
      "name": "__intnl_SystemBoolean_10",
      "type": "System.Boolean",
      "address": 129
    },
    "__intnl_SystemBoolean_20": {
      "name": "__intnl_SystemBoolean_20",
      "type": "System.Boolean",
      "address": 142
    },
    "__intnl_SystemBoolean_30": {
      "name": "__intnl_SystemBoolean_30",
      "type": "System.Boolean",
      "address": 161
    },
    "__intnl_SystemBoolean_40": {
      "name": "__intnl_SystemBoolean_40",
      "type": "System.Boolean",
      "address": 185
    },
    "listenerList": {
      "name": "listenerList",
      "type": "VRC.SDK3.Data.DataList",
      "address": 21
    },
    "__intnl_UnityEngineVector3Array_1": {
      "name": "__intnl_UnityEngineVector3Array_1",
      "type": "UnityEngine.Vector3[]",
      "address": 189
    },
    "__intnl_UnityEngineVector3Array_0": {
      "name": "__intnl_UnityEngineVector3Array_0",
      "type": "UnityEngine.Vector3[]",
      "address": 183
    },
    "__intnl_UnityEngineVector3Array_3": {
      "name": "__intnl_UnityEngineVector3Array_3",
      "type": "UnityEngine.Vector3[]",
      "address": 191
    },
    "__intnl_UnityEngineVector3Array_2": {
      "name": "__intnl_UnityEngineVector3Array_2",
      "type": "UnityEngine.Vector3[]",
      "address": 190
    },
    "__gintnl_SystemUInt32_15": {
      "name": "__gintnl_SystemUInt32_15",
      "type": "System.UInt32",
      "address": 102
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 84
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 81
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 93
    },
    "__intnl_SystemBoolean_8": {
      "name": "__intnl_SystemBoolean_8",
      "type": "System.Boolean",
      "address": 124
    },
    "__intnl_SystemBoolean_9": {
      "name": "__intnl_SystemBoolean_9",
      "type": "System.Boolean",
      "address": 128
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 107
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 110
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 112
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 113
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 116
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 119
    },
    "__intnl_SystemBoolean_6": {
      "name": "__intnl_SystemBoolean_6",
      "type": "System.Boolean",
      "address": 122
    },
    "__intnl_SystemBoolean_7": {
      "name": "__intnl_SystemBoolean_7",
      "type": "System.Boolean",
      "address": 123
    },
    "surftraceMask": {
      "name": "surftraceMask",
      "type": "UnityEngine.LayerMask",
      "address": 10
    },
    "__intnl_SystemObject_5": {
      "name": "__intnl_SystemObject_5",
      "type": "System.Object",
      "address": 178
    },
    "__intnl_VRCUdonUdonBehaviour_0": {
      "name": "__intnl_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 109
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 34
    },
    "__this_VRCUdonUdonBehaviour_3": {
      "name": "__this_VRCUdonUdonBehaviour_3",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 98
    },
    "__this_VRCUdonUdonBehaviour_2": {
      "name": "__this_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 52
    },
    "__this_VRCUdonUdonBehaviour_1": {
      "name": "__this_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 35
    },
    "__this_VRCUdonUdonBehaviour_0": {
      "name": "__this_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 29
    },
    "inkWidth": {
      "name": "inkWidth",
      "type": "System.Single",
      "address": 5
    },
    "__const_SystemString_8": {
      "name": "__const_SystemString_8",
      "type": "System.String",
      "address": 53
    },
    "__intnl_SystemBoolean_18": {
      "name": "__intnl_SystemBoolean_18",
      "type": "System.Boolean",
      "address": 139
    },
    "__intnl_SystemBoolean_28": {
      "name": "__intnl_SystemBoolean_28",
      "type": "System.Boolean",
      "address": 158
    },
    "__intnl_SystemBoolean_38": {
      "name": "__intnl_SystemBoolean_38",
      "type": "System.Boolean",
      "address": 182
    }
  },
  "entryPoints": [
    {
      "name": "get_roundedTrailShader",
      "address": 0
    },
    {
      "name": "get_AllowCallPen",
      "address": 48
    },
    {
      "name": "_start",
      "address": 96
    },
    {
      "name": "_onPlayerJoined",
      "address": 180
    },
    {
      "name": "_onPlayerLeft",
      "address": 644
    },
    {
      "name": "StartUsing",
      "address": 916
    },
    {
      "name": "EndUsing",
      "address": 1520
    },
    {
      "name": "_DisableClearButtonConstraints",
      "address": 2400
    },
    {
      "name": "__0__SetWidth",
      "address": 2556
    },
    {
      "name": "__0__SetMeshLayer",
      "address": 2648
    },
    {
      "name": "__0__SetColliderLayer",
      "address": 2740
    },
    {
      "name": "__0__SetUsingDoubleClick",
      "address": 2832
    },
    {
      "name": "__0__SetEnabledLateSync",
      "address": 2936
    },
    {
      "name": "__0__SetUsingSurftraceMode",
      "address": 3040
    },
    {
      "name": "ResetPen",
      "address": 3144
    },
    {
      "name": "Respawn",
      "address": 3204
    },
    {
      "name": "Clear",
      "address": 3312
    },
    {
      "name": "UndoDraw",
      "address": 3400
    },
    {
      "name": "EraseOwnInk",
      "address": 3576
    },
    {
      "name": "__0_Register",
      "address": 3752
    },
    {
      "name": "OnPenPickup",
      "address": 3992
    },
    {
      "name": "OnPenDrop",
      "address": 4356
    },
    {
      "name": "_TakeOwnership",
      "address": 4720
    },
    {
      "name": "get_InkId",
      "address": 5344
    },
    {
      "name": "_IncrementInkId",
      "address": 5392
    },
    {
      "name": "__0__SendData",
      "address": 5680
    },
    {
      "name": "_onPreSerialization",
      "address": 5768
    },
    {
      "name": "_onDeserialization",
      "address": 5832
    },
    {
      "name": "_onPostSerialization",
      "address": 5956
    },
    {
      "name": "_ClearSyncBuffer",
      "address": 6236
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": -1598075768554875822
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.QvPen_PenManager"
      }
    },
    "2": {
      "address": 2,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "3": {
      "address": 3,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "4": {
      "address": 4,
      "type": "UnityEngine.Gradient",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Gradient",
          "toString": "UnityEngine.Gradient"
        }
      }
    },
    "5": {
      "address": 5,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.005
      }
    },
    "6": {
      "address": 6,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "7": {
      "address": 7,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 9
      }
    },
    "8": {
      "address": 8,
      "type": "UnityEngine.Material",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "9": {
      "address": 9,
      "type": "UnityEngine.Material",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "10": {
      "address": 10,
      "type": "UnityEngine.LayerMask",
      "value": {
        "isSerializable": true,
        "value": {
          "value": -1
        }
      }
    },
    "11": {
      "address": 11,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "12": {
      "address": 12,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "13": {
      "address": 13,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "14": {
      "address": 14,
      "type": "UnityEngine.UI.Text",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "15": {
      "address": 15,
      "type": "TMPro.TextMeshPro",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "16": {
      "address": 16,
      "type": "TMPro.TextMeshProUGUI",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "17": {
      "address": 17,
      "type": "UnityEngine.Shader",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "18": {
      "address": 18,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "19": {
      "address": 19,
      "type": "UnityEngine.Animations.PositionConstraint",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "20": {
      "address": 20,
      "type": "UnityEngine.Animations.RotationConstraint",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "21": {
      "address": 21,
      "type": "VRC.SDK3.Data.DataList",
      "value": {
        "isSerializable": true,
        "value": []
      }
    },
    "22": {
      "address": 22,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "23": {
      "address": 23,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "24": {
      "address": 24,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "25": {
      "address": 25,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "26": {
      "address": 26,
      "type": "UnityEngine.Shader",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "27": {
      "address": 27,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "28": {
      "address": 28,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "29": {
      "address": 29,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "30": {
      "address": 30,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_penManager__param"
      }
    },
    "31": {
      "address": 31,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__Init"
      }
    },
    "32": {
      "address": 32,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "33": {
      "address": 33,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "get_IsUser"
      }
    },
    "34": {
      "address": 34,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_get_IsUser__ret"
      }
    },
    "35": {
      "address": 35,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "36": {
      "address": 36,
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "37": {
      "address": 37,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "StartUsing"
      }
    },
    "38": {
      "address": 38,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "UnityEngine.Animations.PositionConstraint, UnityEngine.AnimationModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "39": {
      "address": 39,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "UnityEngine.Animations.RotationConstraint, UnityEngine.AnimationModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "40": {
      "address": 40,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 624
      }
    },
    "41": {
      "address": 41,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "42": {
      "address": 42,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_onDrop"
      }
    },
    "43": {
      "address": 43,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "44": {
      "address": 44,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "isPickedUp"
      }
    },
    "45": {
      "address": 45,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "46": {
      "address": 46,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1096
      }
    },
    "47": {
      "address": 47,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "48": {
      "address": 48,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "49": {
      "address": 49,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Occupied"
      }
    },
    "50": {
      "address": 50,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1700
      }
    },
    "51": {
      "address": 51,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2184
      }
    },
    "52": {
      "address": 52,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "53": {
      "address": 53,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_DisableClearButtonConstraints"
      }
    },
    "54": {
      "address": 54,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 2.0
      }
    },
    "55": {
      "address": 55,
      "type": "VRC.Udon.Common.Enums.EventTiming",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "56": {
      "address": 56,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "57": {
      "address": 57,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_UpdateInkData"
      }
    },
    "58": {
      "address": 58,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "59": {
      "address": 59,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "60": {
      "address": 60,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "61": {
      "address": 61,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__1_value__param"
      }
    },
    "62": {
      "address": 62,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__SetUseDoubleClick"
      }
    },
    "63": {
      "address": 63,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "64": {
      "address": 64,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__2_value__param"
      }
    },
    "65": {
      "address": 65,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__SetEnabledLateSync"
      }
    },
    "66": {
      "address": 66,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "67": {
      "address": 67,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__3_value__param"
      }
    },
    "68": {
      "address": 68,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__SetUseSurftraceMode"
      }
    },
    "69": {
      "address": 69,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3168
      }
    },
    "70": {
      "address": 70,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3184
      }
    },
    "71": {
      "address": 71,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_Respawn"
      }
    },
    "72": {
      "address": 72,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3292
      }
    },
    "73": {
      "address": 73,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3336
      }
    },
    "74": {
      "address": 74,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_Clear"
      }
    },
    "75": {
      "address": 75,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3512
      }
    },
    "76": {
      "address": 76,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "77": {
      "address": 77,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_UndoDraw"
      }
    },
    "78": {
      "address": 78,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3688
      }
    },
    "79": {
      "address": 79,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_EraseOwnInk"
      }
    },
    "80": {
      "address": 80,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "81": {
      "address": 81,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "82": {
      "address": 82,
      "type": "VRC.SDK3.Data.TokenType",
      "value": {
        "isSerializable": true,
        "value": 15
      }
    },
    "83": {
      "address": 83,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "OnPenPickup"
      }
    },
    "84": {
      "address": 84,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "85": {
      "address": 85,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "OnPenDrop"
      }
    },
    "86": {
      "address": 86,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.GameObject, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "87": {
      "address": 87,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4784
      }
    },
    "88": {
      "address": 88,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "89": {
      "address": 89,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "90": {
      "address": 90,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "91": {
      "address": 91,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 5136
      }
    },
    "92": {
      "address": 92,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 5216
      }
    },
    "93": {
      "address": 93,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "94": {
      "address": 94,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__5_data__param"
      }
    },
    "95": {
      "address": 95,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_targetMode__param"
      }
    },
    "96": {
      "address": 96,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__UnpackData"
      }
    },
    "97": {
      "address": 97,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "98": {
      "address": 98,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "99": {
      "address": 99,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "100": {
      "address": 100,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 5748
      }
    },
    "101": {
      "address": 101,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 5792
      }
    },
    "102": {
      "address": 102,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 5936
      }
    },
    "103": {
      "address": 103,
      "type": "VRC.Udon.Common.SerializationResult",
      "value": {
        "isSerializable": true,
        "value": {
          "success": false,
          "byteCount": 0
        }
      }
    },
    "104": {
      "address": 104,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__6_data__param"
      }
    },
    "105": {
      "address": 105,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__EraseAbandonedInk"
      }
    },
    "106": {
      "address": 106,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6304
      }
    },
    "107": {
      "address": 107,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "108": {
      "address": 108,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "109": {
      "address": 109,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "110": {
      "address": 110,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "111": {
      "address": 111,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "112": {
      "address": 112,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "113": {
      "address": 113,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "114": {
      "address": 114,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "115": {
      "address": 115,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "116": {
      "address": 116,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "117": {
      "address": 117,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "118": {
      "address": 118,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "119": {
      "address": 119,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "120": {
      "address": 120,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "121": {
      "address": 121,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "122": {
      "address": 122,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "123": {
      "address": 123,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "124": {
      "address": 124,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "125": {
      "address": 125,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "126": {
      "address": 126,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "127": {
      "address": 127,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "128": {
      "address": 128,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "129": {
      "address": 129,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "130": {
      "address": 130,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "131": {
      "address": 131,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "132": {
      "address": 132,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "133": {
      "address": 133,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "134": {
      "address": 134,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "135": {
      "address": 135,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "136": {
      "address": 136,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "137": {
      "address": 137,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "138": {
      "address": 138,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "139": {
      "address": 139,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "140": {
      "address": 140,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "141": {
      "address": 141,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "142": {
      "address": 142,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "143": {
      "address": 143,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "144": {
      "address": 144,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "145": {
      "address": 145,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "146": {
      "address": 146,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "147": {
      "address": 147,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "148": {
      "address": 148,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "149": {
      "address": 149,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "150": {
      "address": 150,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "151": {
      "address": 151,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "152": {
      "address": 152,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "153": {
      "address": 153,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "154": {
      "address": 154,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "155": {
      "address": 155,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "156": {
      "address": 156,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "157": {
      "address": 157,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "158": {
      "address": 158,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "159": {
      "address": 159,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "160": {
      "address": 160,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "161": {
      "address": 161,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "162": {
      "address": 162,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "163": {
      "address": 163,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "164": {
      "address": 164,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "165": {
      "address": 165,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "166": {
      "address": 166,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "167": {
      "address": 167,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "168": {
      "address": 168,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "169": {
      "address": 169,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "170": {
      "address": 170,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "171": {
      "address": 171,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "172": {
      "address": 172,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "173": {
      "address": 173,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "174": {
      "address": 174,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "175": {
      "address": 175,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "176": {
      "address": 176,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "177": {
      "address": 177,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "178": {
      "address": 178,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "179": {
      "address": 179,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "180": {
      "address": 180,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "181": {
      "address": 181,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "182": {
      "address": 182,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "183": {
      "address": 183,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "184": {
      "address": 184,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "185": {
      "address": 185,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "186": {
      "address": 186,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "187": {
      "address": 187,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "188": {
      "address": 188,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "189": {
      "address": 189,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "190": {
      "address": 190,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "191": {
      "address": 191,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "192": {
      "address": 192,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SetProgramVariable__SystemString_SystemObject__SystemVoid"
      }
    },
    "193": {
      "address": 193,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEvent__SystemString__SystemVoid"
      }
    },
    "194": {
      "address": 194,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__get_gameObject__UnityEngineGameObject"
      }
    },
    "195": {
      "address": 195,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__IsOwner__UnityEngineGameObject__SystemBoolean"
      }
    },
    "196": {
      "address": 196,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariable__SystemString__SystemObject"
      }
    },
    "197": {
      "address": 197,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomNetworkEvent__VRCUdonCommonInterfacesNetworkEventTarget_SystemString__SystemVoid"
      }
    },
    "198": {
      "address": 198,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__get_isLocal__SystemBoolean"
      }
    },
    "199": {
      "address": 199,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseUtilities.__IsValid__SystemObject__SystemBoolean"
      }
    },
    "200": {
      "address": 200,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__get_transform__UnityEngineTransform"
      }
    },
    "201": {
      "address": 201,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__GetComponent__T"
      }
    },
    "202": {
      "address": 202,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_UnaryNegation__SystemBoolean__SystemBoolean"
      }
    },
    "203": {
      "address": 203,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__SetActive__SystemBoolean__SystemVoid"
      }
    },
    "204": {
      "address": 204,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__GetOwner__UnityEngineGameObject__VRCSDKBaseVRCPlayerApi"
      }
    },
    "205": {
      "address": 205,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObject.__op_Inequality__SystemObject_SystemObject__SystemBoolean"
      }
    },
    "206": {
      "address": 206,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__get_displayName__SystemString"
      }
    },
    "207": {
      "address": 207,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineUIText.__set_text__SystemString__SystemVoid"
      }
    },
    "208": {
      "address": 208,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "TMProTextMeshPro.__set_text__SystemString__SystemVoid"
      }
    },
    "209": {
      "address": 209,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "TMProTextMeshProUGUI.__set_text__SystemString__SystemVoid"
      }
    },
    "210": {
      "address": 210,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__get_Empty__SystemString"
      }
    },
    "211": {
      "address": 211,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineAnimationsPositionConstraint.__set_enabled__SystemBoolean__SystemVoid"
      }
    },
    "212": {
      "address": 212,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineAnimationsRotationConstraint.__set_enabled__SystemBoolean__SystemVoid"
      }
    },
    "213": {
      "address": 213,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEventDelayedSeconds__SystemString_SystemSingle_VRCUdonCommonEnumsEventTiming__SystemVoid"
      }
    },
    "214": {
      "address": 214,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__op_Implicit__UnityEngineObject__VRCSDK3DataDataToken"
      }
    },
    "215": {
      "address": 215,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__Contains__VRCSDK3DataDataToken__SystemBoolean"
      }
    },
    "216": {
      "address": 216,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__Add__VRCSDK3DataDataToken__SystemVoid"
      }
    },
    "217": {
      "address": 217,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__get_Count__SystemInt32"
      }
    },
    "218": {
      "address": 218,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "219": {
      "address": 219,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__TryGetValue__SystemInt32_VRCSDK3DataTokenType_VRCSDK3DataDataTokenRef__SystemBoolean"
      }
    },
    "220": {
      "address": 220,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__get_Reference__SystemObject"
      }
    },
    "221": {
      "address": 221,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "222": {
      "address": 222,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__get_LocalPlayer__VRCSDKBaseVRCPlayerApi"
      }
    },
    "223": {
      "address": 223,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__SetOwner__VRCSDKBaseVRCPlayerApi_UnityEngineGameObject__SystemVoid"
      }
    },
    "224": {
      "address": 224,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__get_IsNetworkSettled__SystemBoolean"
      }
    },
    "225": {
      "address": 225,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__GetPlayerCount__SystemInt32"
      }
    },
    "226": {
      "address": 226,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "227": {
      "address": 227,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__RequestSerialization__SystemVoid"
      }
    },
    "228": {
      "address": 228,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonSerializationResult.__get_success__SystemBoolean"
      }
    },
    "229": {
      "address": 229,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3Array.__ctor__SystemInt32__UnityEngineVector3Array"
      }
    }
  }
}
```