<!-- ci: skip-compile -->

```csharp
using TMPro;
using UdonSharp;
using UnityEngine;
using UnityEngine.UI;
using VRC.SDKBase;
using VRC.Udon.Common;
using VRC.Udon.Common.Interfaces;

#pragma warning disable IDE0044
#pragma warning disable IDE0090, IDE1006

namespace QvPen.UdonScript
{
    [AddComponentMenu("")]
    [DefaultExecutionOrder(20)]
    [UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
    public class QvPen_EraserManager : UdonSharpBehaviour
    {
        [SerializeField]
        private QvPen_Eraser eraser;

        // Layer 9 : Player
        public int inkColliderLayer = 9;

        [SerializeField]
        private GameObject respawnButton;
        [SerializeField]
        private GameObject inUseUI;

        [SerializeField]
        private Text textInUse;
        [SerializeField]
        private TextMeshPro textInUseTMP;
        [SerializeField]
        private TextMeshProUGUI textInUseTMPU;

        private void Start() => eraser._Init(this);

        public override void OnPlayerJoined(VRCPlayerApi player)
        {
            if (Networking.LocalPlayer.IsOwner(eraser.gameObject) && eraser.IsUser)
                SendCustomNetworkEvent(NetworkEventTarget.All, nameof(StartUsing));
        }

        public override void OnPlayerLeft(VRCPlayerApi player)
        {
            if (Networking.IsOwner(eraser.gameObject) && !eraser.IsUser)
                eraser.OnDrop();
        }

        public void StartUsing()
        {
            eraser.isPickedUp = true;

            respawnButton.SetActive(false);
            inUseUI.SetActive(true);

            var owner = Networking.GetOwner(eraser.gameObject);

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
            eraser.isPickedUp = false;

            respawnButton.SetActive(true);
            inUseUI.SetActive(false);

            if (Utilities.IsValid(textInUse))
                textInUse.text = string.Empty;

            if (Utilities.IsValid(textInUseTMP))
                textInUseTMP.text = string.Empty;

            if (Utilities.IsValid(textInUseTMPU))
                textInUseTMPU.text = string.Empty;
        }

        public void ResetEraser() => eraser._Respawn();

        public void Respawn() => eraser._Respawn();

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

                eraser._UnpackData(_syncedData);
            }
        }

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
            => syncedData = _syncedData;

        public override void OnPostSerialization(SerializationResult result)
        {
            isInUseSyncBuffer = false;

            if (result.success)
                eraser.ExecuteEraseInk();
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
  "byteCodeHex": "000000010000000D0000000100000003000000010000000F000000010000000E00000006000000580000000100000003000000010000001000000006000000590000000100000002000000090000000800000002000000010000000D0000000100000034000000060000005A00000001000000030000000100000035000000060000005B000000010000003400000001000000350000000100000033000000060000005C000000010000003300000004000001280000000100000003000000010000000300000001000000360000000900000001000000120000000600000059000000010000000300000001000000130000000100000038000000060000005D0000000100000038000000010000003700000009000000010000003700000001000000330000000900000001000000330000000400000158000000010000001400000001000000150000000100000016000000060000005E0000000100000002000000090000000800000002000000010000000D0000000100000003000000010000003A000000060000005B000000010000003A0000000100000039000000060000005F0000000100000039000000040000022C00000001000000030000000100000003000000010000003B000000090000000100000012000000060000005900000001000000030000000100000013000000010000003D000000060000005D000000010000003D000000010000003C00000009000000010000003C000000010000003900000006000000600000000100000039000000040000026800000001000000030000000100000003000000010000003E00000009000000010000001800000006000000590000000100000002000000090000000800000002000000010000000D0000000100000003000000010000001A000000010000001900000006000000580000000100000005000000010000001B000000060000006100000001000000060000000100000019000000060000006100000001000000030000000100000040000000060000005B0000000100000040000000010000003F0000000600000062000000010000003F000000010000001C0000000100000042000000060000006300000001000000420000000400000354000000010000003F000000010000004100000006000000640000000500000368000000010000001D000000010000004100000009000000010000000700000001000000430000000600000065000000010000004300000004000003A8000000010000000700000001000000410000000600000066000000010000000800000001000000440000000600000065000000010000004400000004000003E8000000010000000800000001000000410000000600000067000000010000000900000001000000450000000600000065000000010000004500000004000004280000000100000009000000010000004100000006000000680000000100000002000000090000000800000002000000010000000D0000000100000003000000010000001A000000010000001B00000006000000580000000100000005000000010000001900000006000000610000000100000006000000010000001B0000000600000061000000010000000700000001000000460000000600000065000000010000004600000004000004E40000000100000047000000060000006900000001000000070000000100000047000000060000006600000001000000080000000100000048000000060000006500000001000000480000000400000534000000010000004900000006000000690000000100000008000000010000004900000006000000670000000100000009000000010000004A0000000600000065000000010000004A0000000400000584000000010000004B00000006000000690000000100000009000000010000004B00000006000000680000000100000002000000090000000800000002000000010000000D00000001000000030000000100000003000000010000004C00000009000000010000001E00000006000000590000000100000002000000090000000800000002000000010000000D00000001000000030000000100000003000000010000004D00000009000000010000001E00000006000000590000000100000002000000090000000800000002000000010000000D0000000100000020000000010000004E000000060000005F000000010000004E000000040000069800000001000000210000000500000AC00000000100000019000000010000001F00000009000000010000000200000009000000080000000200000005000006EC000000010000004F000000060000005A000000010000004F0000000100000020000000060000006A0000000100000020000000010000001F000000060000005F00000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000000D000000010000000A000000010000005000000009000000010000005000000004000007340000000500000758000000010000000A000000060000006B000000010000000A00000001000000500000000900000001000000500000000100000022000000090000000100000002000000090000000800000002000000010000000D000000010000000B0000000100000023000000090000000100000002000000090000000800000002000000010000000D00000001000000250000000500000708000000010000002200000004000007E000000005000007F400000001000000020000000900000008000000020000000100000024000000010000000B0000000900000001000000260000000500000880000000010000000B00000001000000510000000900000001000000030000000100000027000000010000000B00000006000000580000000100000003000000010000002800000006000000590000000100000002000000090000000800000002000000010000000D0000000100000054000000060000006C000000010000005400000001000000290000000100000053000000060000006D000000010000005300000004000008D800000001000000200000000100000053000000060000005F000000010000005300000001000000520000000900000001000000520000000400000914000000010000000C00000001000000520000000600000060000000010000005200000004000009480000000100000019000000010000000C00000009000000010000002A000000060000006E0000000100000002000000090000000800000002000000010000000D000000010000000C000000040000097C00000005000009A0000000010000002C000000010000002B00000001000000240000000900000005000007B80000000100000002000000090000000800000002000000010000000D000000010000002D00000005000007880000000100000023000000010000000B000000090000000100000002000000090000000800000002000000010000000D000000010000002E000000010000000B00000001000000240000000900000005000007B80000000100000002000000090000000800000002000000010000000D000000010000001B000000010000000C00000009000000010000002F0000000100000055000000060000006F00000001000000550000000400000AA400000001000000030000000100000003000000010000005600000009000000010000003000000006000000590000000100000002000000090000000800000002000000010000000D0000000100000031000000010000003200000001000000570000000600000070000000010000005700000001000000240000000900000005000007B8000000010000001B000000010000000C000000090000000100000002000000090000000800000002",
  "byteCodeLength": 2852,
  "symbols": {
    "__intnl_SystemBoolean_13": {
      "name": "__intnl_SystemBoolean_13",
      "type": "System.Boolean",
      "address": 82
    },
    "__intnl_SystemString_1": {
      "name": "__intnl_SystemString_1",
      "type": "System.String",
      "address": 73
    },
    "__intnl_SystemObject_0": {
      "name": "__intnl_SystemObject_0",
      "type": "System.Object",
      "address": 56
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 15
    },
    "_isNetworkSettled": {
      "name": "_isNetworkSettled",
      "type": "System.Boolean",
      "address": 10
    },
    "__gintnl_SystemUInt32_5": {
      "name": "__gintnl_SystemUInt32_5",
      "type": "System.UInt32",
      "address": 46
    },
    "__gintnl_SystemUInt32_4": {
      "name": "__gintnl_SystemUInt32_4",
      "type": "System.UInt32",
      "address": 45
    },
    "__gintnl_SystemUInt32_6": {
      "name": "__gintnl_SystemUInt32_6",
      "type": "System.UInt32",
      "address": 49
    },
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 37
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 33
    },
    "__gintnl_SystemUInt32_3": {
      "name": "__gintnl_SystemUInt32_3",
      "type": "System.UInt32",
      "address": 44
    },
    "__gintnl_SystemUInt32_2": {
      "name": "__gintnl_SystemUInt32_2",
      "type": "System.UInt32",
      "address": 38
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 25
    },
    "__const_SystemBoolean_1": {
      "name": "__const_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 27
    },
    "__0_value__param": {
      "name": "__0_value__param",
      "type": "UnityEngine.Vector3[]",
      "address": 36
    },
    "respawnButton": {
      "name": "respawnButton",
      "type": "UnityEngine.GameObject",
      "address": 5
    },
    "__const_SystemString_5": {
      "name": "__const_SystemString_5",
      "type": "System.String",
      "address": 24
    },
    "onPostSerializationResult": {
      "name": "onPostSerializationResult",
      "type": "VRC.Udon.Common.SerializationResult",
      "address": 47
    },
    "inkColliderLayer": {
      "name": "inkColliderLayer",
      "type": "System.Int32",
      "address": 4
    },
    "__const_VRCUdonCommonInterfacesNetworkEventTarget_0": {
      "name": "__const_VRCUdonCommonInterfacesNetworkEventTarget_0",
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "address": 21
    },
    "textInUse": {
      "name": "textInUse",
      "type": "UnityEngine.UI.Text",
      "address": 7
    },
    "__const_SystemString_10": {
      "name": "__const_SystemString_10",
      "type": "System.String",
      "address": 40
    },
    "__const_SystemString_11": {
      "name": "__const_SystemString_11",
      "type": "System.String",
      "address": 48
    },
    "__intnl_VRCSDKBaseVRCPlayerApi_0": {
      "name": "__intnl_VRCSDKBaseVRCPlayerApi_0",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 52
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__intnl_VRCUdonUdonBehaviour_2": {
      "name": "__intnl_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 62
    },
    "__lcl_text_SystemString_0": {
      "name": "__lcl_text_SystemString_0",
      "type": "System.String",
      "address": 65
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 13
    },
    "__intnl_SystemBoolean_11": {
      "name": "__intnl_SystemBoolean_11",
      "type": "System.Boolean",
      "address": 78
    },
    "__0_data__param": {
      "name": "__0_data__param",
      "type": "UnityEngine.Vector3[]",
      "address": 43
    },
    "__intnl_VRCUdonUdonBehaviour_1": {
      "name": "__intnl_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 59
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 18
    },
    "__intnl_UnityEngineGameObject_2": {
      "name": "__intnl_UnityEngineGameObject_2",
      "type": "UnityEngine.GameObject",
      "address": 64
    },
    "__intnl_UnityEngineGameObject_1": {
      "name": "__intnl_UnityEngineGameObject_1",
      "type": "UnityEngine.GameObject",
      "address": 58
    },
    "__intnl_UnityEngineGameObject_0": {
      "name": "__intnl_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 53
    },
    "__intnl_SystemBoolean_14": {
      "name": "__intnl_SystemBoolean_14",
      "type": "System.Boolean",
      "address": 83
    },
    "__const_SystemObject_0": {
      "name": "__const_SystemObject_0",
      "type": "System.Object",
      "address": 28
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 71
    },
    "__intnl_SystemObject_1": {
      "name": "__intnl_SystemObject_1",
      "type": "System.Object",
      "address": 61
    },
    "__const_SystemString_7": {
      "name": "__const_SystemString_7",
      "type": "System.String",
      "address": 29
    },
    "eraser": {
      "name": "eraser",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 3
    },
    "textInUseTMPU": {
      "name": "textInUseTMPU",
      "type": "TMPro.TextMeshProUGUI",
      "address": 9
    },
    "onPlayerJoinedPlayer": {
      "name": "onPlayerJoinedPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 17
    },
    "__intnl_VRCUdonUdonBehaviour_4": {
      "name": "__intnl_VRCUdonUdonBehaviour_4",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 77
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "__const_SystemString_4": {
      "name": "__const_SystemString_4",
      "type": "System.String",
      "address": 22
    },
    "textInUseTMP": {
      "name": "textInUseTMP",
      "type": "TMPro.TextMeshPro",
      "address": 8
    },
    "isInUseSyncBuffer": {
      "name": "isInUseSyncBuffer",
      "type": "System.Boolean",
      "address": 12
    },
    "__intnl_VRCUdonUdonBehaviour_3": {
      "name": "__intnl_VRCUdonUdonBehaviour_3",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 76
    },
    "__const_SystemString_9": {
      "name": "__const_SystemString_9",
      "type": "System.String",
      "address": 39
    },
    "__intnl_SystemBoolean_12": {
      "name": "__intnl_SystemBoolean_12",
      "type": "System.Boolean",
      "address": 80
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 84
    },
    "__intnl_SystemString_2": {
      "name": "__intnl_SystemString_2",
      "type": "System.String",
      "address": 75
    },
    "onPlayerLeftPlayer": {
      "name": "onPlayerLeftPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 23
    },
    "__0_get_syncedData__ret": {
      "name": "__0_get_syncedData__ret",
      "type": "UnityEngine.Vector3[]",
      "address": 35
    },
    "__lcl_owner_VRCSDKBaseVRCPlayerApi_0": {
      "name": "__lcl_owner_VRCSDKBaseVRCPlayerApi_0",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 63
    },
    "inUseUI": {
      "name": "inUseUI",
      "type": "UnityEngine.GameObject",
      "address": 6
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 16
    },
    "__0__TakeOwnership__ret": {
      "name": "__0__TakeOwnership__ret",
      "type": "System.Boolean",
      "address": 31
    },
    "__intnl_SystemBoolean_15": {
      "name": "__intnl_SystemBoolean_15",
      "type": "System.Boolean",
      "address": 85
    },
    "_syncedData": {
      "name": "_syncedData",
      "type": "UnityEngine.Vector3[]",
      "address": 11
    },
    "__const_SystemString_6": {
      "name": "__const_SystemString_6",
      "type": "System.String",
      "address": 26
    },
    "__0_get_isNetworkSettled__ret": {
      "name": "__0_get_isNetworkSettled__ret",
      "type": "System.Boolean",
      "address": 34
    },
    "__this_UnityEngineGameObject_0": {
      "name": "__this_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 32
    },
    "__intnl_VRCSDKBaseVRCPlayerApi_1": {
      "name": "__intnl_VRCSDKBaseVRCPlayerApi_1",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 79
    },
    "__intnl_VRCUdonUdonBehaviour_5": {
      "name": "__intnl_VRCUdonUdonBehaviour_5",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 86
    },
    "__intnl_SystemBoolean_10": {
      "name": "__intnl_SystemBoolean_10",
      "type": "System.Boolean",
      "address": 74
    },
    "__intnl_UnityEngineVector3Array_1": {
      "name": "__intnl_UnityEngineVector3Array_1",
      "type": "UnityEngine.Vector3[]",
      "address": 87
    },
    "__intnl_UnityEngineVector3Array_0": {
      "name": "__intnl_UnityEngineVector3Array_0",
      "type": "UnityEngine.Vector3[]",
      "address": 81
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 50
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 41
    },
    "__intnl_SystemBoolean_8": {
      "name": "__intnl_SystemBoolean_8",
      "type": "System.Boolean",
      "address": 70
    },
    "__intnl_SystemBoolean_9": {
      "name": "__intnl_SystemBoolean_9",
      "type": "System.Boolean",
      "address": 72
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 51
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 55
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 57
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 60
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 66
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 67
    },
    "__intnl_SystemBoolean_6": {
      "name": "__intnl_SystemBoolean_6",
      "type": "System.Boolean",
      "address": 68
    },
    "__intnl_SystemBoolean_7": {
      "name": "__intnl_SystemBoolean_7",
      "type": "System.Boolean",
      "address": 69
    },
    "__intnl_VRCUdonUdonBehaviour_0": {
      "name": "__intnl_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 54
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 19
    },
    "__this_VRCUdonUdonBehaviour_2": {
      "name": "__this_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 42
    },
    "__this_VRCUdonUdonBehaviour_1": {
      "name": "__this_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 20
    },
    "__this_VRCUdonUdonBehaviour_0": {
      "name": "__this_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 14
    },
    "__const_SystemString_8": {
      "name": "__const_SystemString_8",
      "type": "System.String",
      "address": 30
    }
  },
  "entryPoints": [
    {
      "name": "_start",
      "address": 0
    },
    {
      "name": "_onPlayerJoined",
      "address": 84
    },
    {
      "name": "_onPlayerLeft",
      "address": 364
    },
    {
      "name": "StartUsing",
      "address": 636
    },
    {
      "name": "EndUsing",
      "address": 1084
    },
    {
      "name": "ResetEraser",
      "address": 1432
    },
    {
      "name": "Respawn",
      "address": 1504
    },
    {
      "name": "_TakeOwnership",
      "address": 1576
    },
    {
      "name": "__0__SendData",
      "address": 2396
    },
    {
      "name": "_onPreSerialization",
      "address": 2484
    },
    {
      "name": "_onDeserialization",
      "address": 2548
    },
    {
      "name": "_onPostSerialization",
      "address": 2612
    },
    {
      "name": "_ClearSyncBuffer",
      "address": 2744
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": -5465713458096539185
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.QvPen_EraserManager"
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 9
      }
    },
    "5": {
      "address": 5,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "6": {
      "address": 6,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "7": {
      "address": 7,
      "type": "UnityEngine.UI.Text",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "8": {
      "address": 8,
      "type": "TMPro.TextMeshPro",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "9": {
      "address": 9,
      "type": "TMPro.TextMeshProUGUI",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "10": {
      "address": 10,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "11": {
      "address": 11,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "12": {
      "address": 12,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "13": {
      "address": 13,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "14": {
      "address": 14,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "15": {
      "address": 15,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_eraserManager__param"
      }
    },
    "16": {
      "address": 16,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__Init"
      }
    },
    "17": {
      "address": 17,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "get_IsUser"
      }
    },
    "19": {
      "address": 19,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_get_IsUser__ret"
      }
    },
    "20": {
      "address": 20,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "21": {
      "address": 21,
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "StartUsing"
      }
    },
    "23": {
      "address": 23,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "24": {
      "address": 24,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_onDrop"
      }
    },
    "25": {
      "address": 25,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "26": {
      "address": 26,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "isPickedUp"
      }
    },
    "27": {
      "address": 27,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "28": {
      "address": 28,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "29": {
      "address": 29,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Occupied"
      }
    },
    "30": {
      "address": 30,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_Respawn"
      }
    },
    "31": {
      "address": 31,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "32": {
      "address": 32,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.GameObject, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "33": {
      "address": 33,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1640
      }
    },
    "34": {
      "address": 34,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "35": {
      "address": 35,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "36": {
      "address": 36,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "37": {
      "address": 37,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1992
      }
    },
    "38": {
      "address": 38,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2072
      }
    },
    "39": {
      "address": 39,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__4_data__param"
      }
    },
    "40": {
      "address": 40,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__UnpackData"
      }
    },
    "41": {
      "address": 41,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "42": {
      "address": 42,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "43": {
      "address": 43,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "44": {
      "address": 44,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2464
      }
    },
    "45": {
      "address": 45,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2508
      }
    },
    "46": {
      "address": 46,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2592
      }
    },
    "47": {
      "address": 47,
      "type": "VRC.Udon.Common.SerializationResult",
      "value": {
        "isSerializable": true,
        "value": {
          "success": false,
          "byteCount": 0
        }
      }
    },
    "48": {
      "address": 48,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ExecuteEraseInk"
      }
    },
    "49": {
      "address": 49,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2812
      }
    },
    "50": {
      "address": 50,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "51": {
      "address": 51,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "52": {
      "address": 52,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "53": {
      "address": 53,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "54": {
      "address": 54,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "55": {
      "address": 55,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "56": {
      "address": 56,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "57": {
      "address": 57,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "58": {
      "address": 58,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "59": {
      "address": 59,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "62": {
      "address": 62,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "63": {
      "address": 63,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "64": {
      "address": 64,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "65": {
      "address": 65,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "68": {
      "address": 68,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "69": {
      "address": 69,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "70": {
      "address": 70,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "71": {
      "address": 71,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "72": {
      "address": 72,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "73": {
      "address": 73,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "74": {
      "address": 74,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "75": {
      "address": 75,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "76": {
      "address": 76,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "77": {
      "address": 77,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "78": {
      "address": 78,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "79": {
      "address": 79,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "80": {
      "address": 80,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "81": {
      "address": 81,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "82": {
      "address": 82,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "83": {
      "address": 83,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "84": {
      "address": 84,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "85": {
      "address": 85,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "86": {
      "address": 86,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "87": {
      "address": 87,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "88": {
      "address": 88,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SetProgramVariable__SystemString_SystemObject__SystemVoid"
      }
    },
    "89": {
      "address": 89,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEvent__SystemString__SystemVoid"
      }
    },
    "90": {
      "address": 90,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__get_LocalPlayer__VRCSDKBaseVRCPlayerApi"
      }
    },
    "91": {
      "address": 91,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__get_gameObject__UnityEngineGameObject"
      }
    },
    "92": {
      "address": 92,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__IsOwner__UnityEngineGameObject__SystemBoolean"
      }
    },
    "93": {
      "address": 93,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariable__SystemString__SystemObject"
      }
    },
    "94": {
      "address": 94,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomNetworkEvent__VRCUdonCommonInterfacesNetworkEventTarget_SystemString__SystemVoid"
      }
    },
    "95": {
      "address": 95,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__IsOwner__UnityEngineGameObject__SystemBoolean"
      }
    },
    "96": {
      "address": 96,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_UnaryNegation__SystemBoolean__SystemBoolean"
      }
    },
    "97": {
      "address": 97,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__SetActive__SystemBoolean__SystemVoid"
      }
    },
    "98": {
      "address": 98,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__GetOwner__UnityEngineGameObject__VRCSDKBaseVRCPlayerApi"
      }
    },
    "99": {
      "address": 99,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObject.__op_Inequality__SystemObject_SystemObject__SystemBoolean"
      }
    },
    "100": {
      "address": 100,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__get_displayName__SystemString"
      }
    },
    "101": {
      "address": 101,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseUtilities.__IsValid__SystemObject__SystemBoolean"
      }
    },
    "102": {
      "address": 102,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineUIText.__set_text__SystemString__SystemVoid"
      }
    },
    "103": {
      "address": 103,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "TMProTextMeshPro.__set_text__SystemString__SystemVoid"
      }
    },
    "104": {
      "address": 104,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "TMProTextMeshProUGUI.__set_text__SystemString__SystemVoid"
      }
    },
    "105": {
      "address": 105,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__get_Empty__SystemString"
      }
    },
    "106": {
      "address": 106,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__SetOwner__VRCSDKBaseVRCPlayerApi_UnityEngineGameObject__SystemVoid"
      }
    },
    "107": {
      "address": 107,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__get_IsNetworkSettled__SystemBoolean"
      }
    },
    "108": {
      "address": 108,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__GetPlayerCount__SystemInt32"
      }
    },
    "109": {
      "address": 109,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "110": {
      "address": 110,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__RequestSerialization__SystemVoid"
      }
    },
    "111": {
      "address": 111,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonSerializationResult.__get_success__SystemBoolean"
      }
    },
    "112": {
      "address": 112,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3Array.__ctor__SystemInt32__UnityEngineVector3Array"
      }
    }
  }
}
```
