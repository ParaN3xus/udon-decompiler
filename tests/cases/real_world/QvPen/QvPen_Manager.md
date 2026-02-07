<!-- ci: skip-compile -->

```csharp
﻿using UdonSharp;
using UnityEngine;
using VRC.SDK3.Data;
using VRC.SDKBase;
using Utilities = VRC.SDKBase.Utilities;

#pragma warning disable IDE0044
#pragma warning disable IDE0090, IDE1006

namespace QvPen.UdonScript
{
    [AddComponentMenu("")]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_Manager : UdonSharpBehaviour
    {
        private readonly DataDictionary penDict = new DataDictionary();
        private readonly DataDictionary inkDictMap = new DataDictionary();
        private readonly DataList callablePenList = new DataList();

        public void Register(int penId, QvPen_Pen pen)
        {
            if (penDict.ContainsKey(penId))
                return;

            penDict[penId] = pen;
            inkDictMap[penId] = new DataDictionary();

            if (pen.AllowCallPen)
                callablePenList.Add(pen);
        }

        #region Call Pen

        private void Update()
        {
            if (!Input.anyKeyDown)
                return;

            if (Input.GetKeyDown(KeyCode.Q) && Input.GetKey(KeyCode.Tab))
                CallPen();
        }

        private QvPen_Pen lastUsedPen;

        public void SetLastUsedPen(QvPen_Pen pen)
        {
            lastUsedPen = pen;
        }

        private void CallPen()
        {
            var pen = GetPen();

            if (!Utilities.IsValid(pen))
                return;

            var x = Networking.LocalPlayer.GetTrackingData(VRCPlayerApi.TrackingDataType.Head);
            var forward = x.rotation * Vector3.forward;
            pen.transform.position = x.position + 0.5f * forward;
            pen.transform.LookAt(x.position + forward);
        }

        private QvPen_Pen GetPen()
        {
            var pickupR = Networking.LocalPlayer.GetPickupInHand(VRC_Pickup.PickupHand.Right);

            if (Utilities.IsValid(pickupR))
                return null;

            var pickupL = Networking.LocalPlayer.GetPickupInHand(VRC_Pickup.PickupHand.Left);

            if (Utilities.IsValid(pickupL))
                return null;

            if (Utilities.IsValid(lastUsedPen) && lastUsedPen.AllowCallPen && !lastUsedPen.isHeld)
            {
                lastUsedPen._TakeOwnership();

                return lastUsedPen;
            }

            if (callablePenList.Count == 0)
                return null;

            var indexList = new int[callablePenList.Count];

            for (int i = 0, n = callablePenList.Count; i < n; i++)
                indexList[i] = i;

            Utilities.ShuffleArray(indexList);

            for (int i = 0, n = indexList.Length; i < n; i++)
            {
                var index = indexList[i];

                if (!callablePenList.TryGetValue(index, TokenType.Reference, out var penToken))
                    continue;

                var pen = (QvPen_Pen)penToken.Reference;

                if (!pen.isHeld)
                {
                    pen._TakeOwnership();

                    lastUsedPen = pen;
                    return pen;
                }
            }

            return null;
        }

        #endregion

        #region Manage Ink

        public bool HasInk(int penId, int inkId)
        {
            if (!inkDictMap.TryGetValue(penId, TokenType.DataDictionary, out var inkDictToken))
                return false;

            if (!inkDictToken.DataDictionary.ContainsKey(inkId))
                return false;

            return true;
        }

        public void SetInk(int penId, int inkId, GameObject inkInstance)
        {
            inkDictMap[penId].DataDictionary[inkId] = inkInstance;
        }

        public bool RemoveInk(int penId, int inkId)
        {
            if (!HasInk(penId, inkId))
                return false;

            var inkDict = inkDictMap[penId].DataDictionary;

            if (!inkDict.TryGetValue(inkId, TokenType.Reference, out var inkToken))
                return false;

            var ink = (GameObject)inkToken.Reference;

            if (!Utilities.IsValid(ink))
            {
                inkDict.Remove(inkId);
                return false;
            }

            Destroy(ink.GetComponentInChildren<MeshCollider>(true).sharedMesh);
            Destroy(ink);

            inkDict.Remove(inkId);

            return true;
        }

        public bool RemoveUserInk(int penId, Vector3 ownerIdVector)
        {
            var inkDict = inkDictMap[penId].DataDictionary;

            var inkIdList = inkDict.GetKeys();

            var removedInkIdList = new DataList();

            var ownerId = QvPenUtilities.EulerAnglesToPlayerId(ownerIdVector);

            for (int i = 0, n = inkIdList.Count; i < n; i++)
            {
                if (!inkIdList.TryGetValue(i, TokenType.Int, out var inkIdToken))
                    continue;

                if (!inkDict.TryGetValue(inkIdToken, TokenType.Reference, out var inkToken))
                    continue;

                var ink = (GameObject)inkToken.Reference;

                if (!Utilities.IsValid(ink))
                {
                    removedInkIdList.Add(inkIdToken);
                    continue;
                }

                if (!QvPenUtilities.TryGetIdFromInk(ink, out var _discard1, out var _discard2, out var inkOwnerIdVector))
                    continue;

                var inkOwnerId = QvPenUtilities.EulerAnglesToPlayerId(inkOwnerIdVector);

                if (inkOwnerId != ownerId)
                    continue;

                Destroy(ink.GetComponentInChildren<MeshCollider>(true).sharedMesh);
                Destroy(ink);

                removedInkIdList.Add(inkIdToken);
            }

            for (int i = 0, n = removedInkIdList.Count; i < n; i++)
            {
                if (!removedInkIdList.TryGetValue(i, TokenType.Int, out var inkIdToken))
                    continue;

                inkDict.Remove(inkIdToken);
            }

            return true;
        }

        public void Clear(int penId)
        {
            if (!inkDictMap.TryGetValue(penId, TokenType.DataDictionary, out var inkDictToken))
                return;

            var inkDict = inkDictToken.DataDictionary;
            var inkTokens = inkDict.GetValues();

            for (int i = 0, n = inkTokens.Count; i < n; i++)
            {
                if (!inkTokens.TryGetValue(i, TokenType.Reference, out var inkToken))
                    continue;

                var ink = (GameObject)inkToken.Reference;

                if (!Utilities.IsValid(ink))
                    continue;

                Destroy(ink.GetComponentInChildren<MeshCollider>(true).sharedMesh);
                Destroy(ink);
            }

            inkDict.Clear();
        }

        #endregion

        #region Log

        private const string appName = nameof(QvPen_Manager);

        private void Log(object o) => Debug.Log($"{logPrefix}{o}", this);
        private void Warning(object o) => Debug.LogWarning($"{logPrefix}{o}", this);
        private void Error(object o) => Debug.LogError($"{logPrefix}{o}", this);

        private readonly Color logColor = new Color(0xf2, 0x7d, 0x4a, 0xff) / 0xff;
        private string ColorBeginTag(Color c) => $"<color=\"#{ToHtmlStringRGB(c)}\">";
        private const string ColorEndTag = "</color>";

        private string _logPrefix;
        private string logPrefix
            => !string.IsNullOrEmpty(_logPrefix)
                ? _logPrefix : (_logPrefix = $"[{ColorBeginTag(logColor)}{nameof(QvPen)}.{nameof(QvPen.Udon)}.{appName}{ColorEndTag}] ");

        private static string ToHtmlStringRGB(Color c)
        {
            c *= 0xff;
            return $"{Mathf.RoundToInt(c.r):x2}{Mathf.RoundToInt(c.g):x2}{Mathf.RoundToInt(c.b):x2}";
        }

        #endregion
    }
}
```

```json
{
  "byteCodeHex": "000000010000000B0000000100000009000000010000005F00000006000000FD0000000100000003000000010000005F000000010000006000000006000000FE0000000100000060000000040000006400000001000000020000000900000008000000020000000100000009000000010000006100000006000000FD000000010000000A000000010000006200000006000000FF00000001000000030000000100000061000000010000006200000006000001000000000100000009000000010000006300000006000000FD000000010000006400000006000001010000000100000064000000010000006500000006000001020000000100000004000000010000006300000001000000650000000600000100000000010000000A000000010000000A000000010000006600000009000000010000000C0000000600000103000000010000000A000000010000000D000000010000006800000006000001040000000100000068000000010000006700000009000000010000006700000004000001B4000000010000000A000000010000006900000006000000FF0000000100000005000000010000006900000006000001050000000100000002000000090000000800000002000000010000000B000000010000006A0000000600000106000000010000006A00000004000001F8000000050000020C0000000100000002000000090000000800000002000000010000000E000000010000006B0000000600000107000000010000006B000000040000024C000000010000000F000000010000006B0000000600000108000000010000006B000000040000026C000000010000001000000005000002B80000000100000002000000090000000800000002000000010000000B00000001000000110000000100000006000000090000000100000002000000090000000800000002000000010000000B000000010000001200000005000004940000000100000013000000010000006C00000009000000010000006C000000010000006D0000000600000109000000010000006D000000040000030C00000005000003200000000100000002000000090000000800000002000000010000006F000000060000010A000000010000006F0000000100000014000000010000006E000000060000010B000000010000006E0000000100000071000000060000010C000000010000007100000001000000150000000100000070000000060000010D000000010000006C0000000100000072000000060000010E000000010000006E0000000100000073000000060000010F00000001000000160000000100000070000000010000007400000006000001100000000100000073000000010000007400000001000000750000000600000111000000010000007200000001000000750000000600000112000000010000006C0000000100000076000000060000010E000000010000006E0000000100000077000000060000010F00000001000000770000000100000070000000010000007800000006000001110000000100000076000000010000007800000006000001130000000100000002000000090000000800000002000000010000000B000000010000007A000000060000010A000000010000007A0000000100000017000000010000007900000006000001140000000100000079000000010000007B0000000600000109000000010000007B000000040000051400000001000000180000000100000013000000090000000100000002000000090000000800000002000000010000007D000000060000010A000000010000007D0000000100000019000000010000007C0000000600000114000000010000007C000000010000007E0000000600000109000000010000007E0000000400000594000000010000001800000001000000130000000900000001000000020000000900000008000000020000000100000006000000010000008000000006000001090000000100000080000000040000063000000001000000060000000100000006000000010000008100000009000000010000000C00000006000001030000000100000006000000010000000D00000001000000830000000600000104000000010000008300000001000000820000000900000001000000820000000100000080000000090000000100000080000000010000007F00000009000000010000007F00000004000006CC00000001000000060000000100000006000000010000008400000009000000010000001A00000006000001030000000100000006000000010000001B0000000100000086000000060000010400000001000000860000000100000085000000090000000100000085000000010000007F0000000600000115000000010000007F000000040000076400000001000000060000000100000006000000010000008B00000009000000010000001C00000006000001030000000100000006000000010000001D000000010000008C0000000600000104000000010000008C000000010000008800000009000000010000000600000001000000130000000900000001000000020000000900000008000000020000000100000005000000010000008700000006000001160000000100000087000000010000001E00000001000000880000000600000117000000010000008800000004000007D4000000010000001800000001000000130000000900000001000000020000000900000008000000020000000100000005000000010000008A0000000600000116000000010000008A00000001000000890000000600000118000000010000001E000000010000008D000000090000000100000005000000010000008E0000000600000116000000010000008D000000010000008E000000010000008F0000000600000119000000010000008F00000004000008A80000000100000089000000010000008D000000010000008D000000060000011A000000010000008D000000010000001F000000010000008D000000060000011B00000005000008300000000100000089000000060000011C000000010000001E000000010000008D000000090000000100000089000000010000008E000000060000011D000000010000008D000000010000008E000000010000008F0000000600000119000000010000008F0000000400000AC40000000100000089000000010000008D0000000100000090000000060000011E00000001000000050000000100000090000000010000002000000001000000910000000100000092000000060000011F0000000100000092000000040000097C00000005000009840000000500000A9C0000000100000091000000010000008C0000000600000120000000010000008C0000000100000093000000090000000100000093000000010000001A00000006000001030000000100000093000000010000001B00000001000000950000000600000104000000010000009500000001000000940000000900000001000000940000000400000A140000000500000A9C0000000100000093000000010000001C00000006000001030000000100000093000000010000001D000000010000009700000006000001040000000100000097000000010000009600000009000000010000009300000001000000060000000900000001000000930000000100000013000000090000000100000002000000090000000800000002000000010000008D000000010000001F000000010000008D000000060000011B00000005000008E4000000010000001800000001000000130000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000000B0000000100000022000000010000009800000006000000FD0000000100000004000000010000009800000001000000240000000100000099000000010000009A0000000600000121000000010000009A0000000400000B680000000500000B90000000010000002500000001000000210000000900000001000000020000000900000008000000020000000100000099000000010000009B00000006000001220000000100000023000000010000009C00000006000000FD000000010000009B000000010000009C000000010000009D00000006000000FE000000010000009D0000000400000BF80000000500000C2000000001000000250000000100000021000000090000000100000002000000090000000800000002000000010000002600000001000000210000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000000B0000000100000027000000010000009E00000006000000FD0000000100000004000000010000009E000000010000009F0000000600000123000000010000009F00000001000000A00000000600000122000000010000002800000001000000A100000006000000FD000000010000002900000001000000A200000006000000FF00000001000000A000000001000000A100000001000000A200000006000001000000000100000002000000090000000800000002000000010000000B000000010000002D000000010000002B000000010000002200000009000000010000002C0000000100000023000000090000000500000B0800000001000000210000000400000D700000000500000D980000000100000025000000010000002A000000090000000100000002000000090000000800000002000000010000002B00000001000000A400000006000000FD000000010000000400000001000000A400000001000000A5000000060000012300000001000000A500000001000000A30000000600000122000000010000002C00000001000000A600000006000000FD00000001000000A300000001000000A6000000010000002000000001000000A700000001000000A8000000060000012100000001000000A80000000400000E480000000500000E700000000100000025000000010000002A00000009000000010000000200000009000000080000000200000001000000A700000001000000AA000000060000012000000001000000AA00000001000000A90000000900000001000000A900000001000000AB000000060000010900000001000000AB0000000400000ECC0000000500000F2C000000010000002C00000001000000B100000006000000FD00000001000000A300000001000000B100000001000000B200000006000001240000000100000025000000010000002A00000009000000010000000200000009000000080000000200000001000000A900000001000000AC000000060000012500000001000000AC0000000100000026000000010000002E00000001000000AD000000060000012600000001000000AD00000001000000AE000000060000012700000001000000AE00000001000000AF0000000900000001000000AF000000060000012800000001000000A900000001000000B00000000900000001000000B00000000600000128000000010000002C00000001000000B100000006000000FD00000001000000A300000001000000B100000001000000B200000006000001240000000100000026000000010000002A0000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000000B000000010000003000000001000000B400000006000000FD000000010000000400000001000000B400000001000000B5000000060000012300000001000000B500000001000000B3000000060000012200000001000000B300000001000000B6000000060000012900000001000000B7000000060000012A000000010000003200000001000000310000000100000034000000090000000500001BA4000000010000003300000001000000B800000009000000010000001E00000001000000B90000000900000001000000B600000001000000BA000000060000011600000001000000B900000001000000BA00000001000000BB000000060000011900000001000000BB000000040000147C00000001000000B600000001000000B9000000010000003500000001000000BC00000001000000BD000000060000011F00000001000000BD000000040000119C00000005000011A4000000050000145400000001000000B300000001000000BC000000010000002000000001000000BE00000001000000BF000000060000012100000001000000BF00000004000011EC00000005000011F4000000050000145400000001000000BE00000001000000C1000000060000012000000001000000C100000001000000C00000000900000001000000C000000001000000C2000000060000010900000001000000C20000000400001250000000050000127000000001000000B700000001000000BC00000006000001050000000500001454000000010000003600000001000000C000000001000000380000000900000001000000C300000001000000390000000900000001000000C4000000010000003A0000000900000001000000C5000000010000003B000000090000000500001CFC000000010000003900000001000000C300000009000000010000003A00000001000000C400000009000000010000003B00000001000000C50000000900000001000000370000000400001324000000050000132C0000000500001454000000010000003C00000001000000C50000000100000034000000090000000500001BA4000000010000003300000001000000C60000000900000001000000C600000001000000B800000001000000C7000000060000012B00000001000000C7000000040000139C000000050000145400000001000000C000000001000000C8000000060000012500000001000000C80000000100000026000000010000002E00000001000000C9000000060000012600000001000000C900000001000000CA000000060000012700000001000000CA00000001000000CB0000000900000001000000CB000000060000012800000001000000C000000001000000CC0000000900000001000000CC000000060000012800000001000000B700000001000000BC000000060000010500000001000000B9000000010000001F00000001000000B9000000060000011B0000000500001124000000010000001E00000001000000B90000000900000001000000B700000001000000BA000000060000011600000001000000B900000001000000BA00000001000000BB000000060000011900000001000000BB000000040000157000000001000000B700000001000000B9000000010000003500000001000000BC00000001000000BD000000060000011F00000001000000BD00000004000015200000000500001528000000050000154800000001000000B300000001000000BC00000001000000BF000000060000012400000001000000B9000000010000001F00000001000000B9000000060000011B00000005000014A80000000100000026000000010000002F0000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000000B000000010000003D00000001000000CD00000006000000FD000000010000000400000001000000CD000000010000002400000001000000CE00000001000000CF000000060000012100000001000000CF00000004000016140000000500001628000000010000000200000009000000080000000200000001000000CE00000001000000D0000000060000012200000001000000D000000001000000D1000000060000012C000000010000001E00000001000000D20000000900000001000000D100000001000000D3000000060000011600000001000000D200000001000000D300000001000000D4000000060000011900000001000000D4000000040000183000000001000000D100000001000000D2000000010000002000000001000000D500000001000000D6000000060000011F00000001000000D600000004000016FC0000000500001704000000050000180800000001000000D500000001000000D8000000060000012000000001000000D800000001000000D70000000900000001000000D700000001000000D9000000060000010900000001000000D900000004000017600000000500001768000000050000180800000001000000D700000001000000DA000000060000012500000001000000DA0000000100000026000000010000002E00000001000000DB000000060000012600000001000000DB00000001000000DC000000060000012700000001000000DC00000001000000DD0000000900000001000000DD000000060000012800000001000000D700000001000000DE0000000900000001000000DE000000060000012800000001000000D2000000010000001F00000001000000D2000000060000011B000000050000168400000001000000D0000000060000012D0000000100000002000000090000000800000002000000010000000B00000001000000400000000500001A3C000000010000003F0000000100000041000000010000003E00000001000000DF000000060000012E000000010000004200000001000000E00000000900000001000000DF00000001000000E0000000060000012F0000000100000002000000090000000800000002000000010000000B00000001000000440000000500001A3C000000010000003F0000000100000041000000010000004300000001000000E1000000060000012E000000010000004500000001000000E20000000900000001000000E100000001000000E200000006000001300000000100000002000000090000000800000002000000010000000B00000001000000470000000500001A3C000000010000003F0000000100000041000000010000004600000001000000E3000000060000012E000000010000004800000001000000E40000000900000001000000E300000001000000E400000006000001310000000100000002000000090000000800000002000000010000000B000000010000004C000000010000004A000000010000004E000000090000000500001FA4000000010000004B000000010000004D000000010000004900000006000001320000000100000002000000090000000800000002000000010000000B000000010000000800000001000000E5000000060000013300000001000000E500000001000000E6000000060000011500000001000000E60000000400001A9800000001000000080000000100000041000000090000000500001B9000000001000000500000000100000007000000010000004A0000000900000005000019DC000000010000004F000000010000001E00000001000000490000000600000134000000010000004F000000010000001F00000001000000510000000600000134000000010000004F000000010000005200000001000000530000000600000134000000010000004F000000010000005400000001000000550000000600000134000000010000004F0000000100000056000000010000005700000006000001340000000100000058000000010000004F00000001000000080000000600000135000000010000000800000001000000410000000900000001000000020000000900000008000000020000000100000034000000010000005900000001000000340000000600000136000000010000003400000001000000E7000000060000013700000001000000E700000001000000E80000000600000138000000010000003400000001000000E9000000060000013900000001000000E900000001000000EA000000060000013800000001000000EA000000010000005A00000001000000EB000000060000013A00000001000000E800000001000000EB00000001000000EC000000060000011B000000010000003400000001000000ED000000060000013B00000001000000ED00000001000000EE000000060000013800000001000000EE000000010000005B00000001000000EF000000060000013A00000001000000EC00000001000000EF0000000100000033000000060000011B00000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000003800000001000000F0000000060000010900000001000000F00000000400001D2C0000000500001D90000000010000005C000000010000003900000009000000010000005C000000010000003A00000009000000010000005C000000010000003B0000000900000001000000250000000100000037000000090000000100000002000000090000000800000002000000010000003800000001000000F1000000060000012500000001000000F100000001000000F2000000060000013C00000001000000F2000000010000005200000001000000F3000000060000011900000001000000F30000000400001E54000000010000005C000000010000003900000009000000010000005C000000010000003A00000009000000010000005C000000010000003B0000000900000001000000250000000100000037000000090000000100000002000000090000000800000002000000010000003800000001000000F5000000060000012500000001000000F5000000010000001F00000001000000F4000000060000013D00000001000000F400000001000000F6000000060000010900000001000000F60000000400001EBC0000000500001F20000000010000005C000000010000003900000009000000010000005C000000010000003A00000009000000010000005C000000010000003B000000090000000100000025000000010000003700000009000000010000000200000009000000080000000200000001000000F40000000100000039000000060000013E00000001000000F4000000010000003A000000060000013F00000001000000F4000000010000003B0000000600000140000000010000002600000001000000370000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000004E000000010000005D000000010000004E0000000600000141000000010000004E00000001000000F7000000060000014200000001000000F700000001000000F80000000600000138000000010000004E00000001000000F9000000060000014300000001000000F900000001000000FA0000000600000138000000010000004E00000001000000FB000000060000014400000001000000FB00000001000000FC0000000600000138000000010000005E00000001000000F800000001000000FA00000001000000FC000000010000004D000000060000014500000001000000020000000900000008000000020000000100000002000000090000000800000002",
  "byteCodeLength": 8364,
  "symbols": {
    "__intnl_SystemBoolean_13": {
      "name": "__intnl_SystemBoolean_13",
      "type": "System.Boolean",
      "address": 146
    },
    "__intnl_SystemBoolean_23": {
      "name": "__intnl_SystemBoolean_23",
      "type": "System.Boolean",
      "address": 191
    },
    "__intnl_SystemBoolean_33": {
      "name": "__intnl_SystemBoolean_33",
      "type": "System.Boolean",
      "address": 243
    },
    "__intnl_VRCSDK3DataDataToken_7": {
      "name": "__intnl_VRCSDK3DataDataToken_7",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 156
    },
    "__1_penId__param": {
      "name": "__1_penId__param",
      "type": "System.Int32",
      "address": 34
    },
    "__lcl__discard2_UnityEngineVector3_0": {
      "name": "__lcl__discard2_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 196
    },
    "__lcl_inkIdToken_VRCSDK3DataDataToken_0": {
      "name": "__lcl_inkIdToken_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 188
    },
    "__intnl_SystemString_1": {
      "name": "__intnl_SystemString_1",
      "type": "System.String",
      "address": 225
    },
    "__6__intnlparam": {
      "name": "__6__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 59
    },
    "__8__intnlparam": {
      "name": "__8__intnlparam",
      "type": "UnityEngine.Color",
      "address": 78
    },
    "__lcl_forward_UnityEngineVector3_0": {
      "name": "__lcl_forward_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 112
    },
    "__intnl_SystemSingle_0": {
      "name": "__intnl_SystemSingle_0",
      "type": "System.Single",
      "address": 231
    },
    "__intnl_SystemObject_0": {
      "name": "__intnl_SystemObject_0",
      "type": "System.Object",
      "address": 104
    },
    "lastUsedPen": {
      "name": "lastUsedPen",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 6
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 12
    },
    "__lcl_ink_UnityEngineGameObject_0": {
      "name": "__lcl_ink_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 169
    },
    "__lcl_ink_UnityEngineGameObject_1": {
      "name": "__lcl_ink_UnityEngineGameObject_1",
      "type": "UnityEngine.GameObject",
      "address": 192
    },
    "__lcl_ink_UnityEngineGameObject_2": {
      "name": "__lcl_ink_UnityEngineGameObject_2",
      "type": "UnityEngine.GameObject",
      "address": 215
    },
    "__intnl_UnityEngineTransform_0": {
      "name": "__intnl_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 114
    },
    "__const_SystemSingle_0": {
      "name": "__const_SystemSingle_0",
      "type": "System.Single",
      "address": 22
    },
    "__0_ownerIdVector__param": {
      "name": "__0_ownerIdVector__param",
      "type": "UnityEngine.Vector3",
      "address": 49
    },
    "__0___0_ColorBeginTag__ret": {
      "name": "__0___0_ColorBeginTag__ret",
      "type": "System.String",
      "address": 73
    },
    "__lcl_indexList_SystemInt32Array_0": {
      "name": "__lcl_indexList_SystemInt32Array_0",
      "type": "System.Int32[]",
      "address": 137
    },
    "__lcl_inkOwnerId_SystemInt32_0": {
      "name": "__lcl_inkOwnerId_SystemInt32_0",
      "type": "System.Int32",
      "address": 198
    },
    "__intnl_SystemBoolean_16": {
      "name": "__intnl_SystemBoolean_16",
      "type": "System.Boolean",
      "address": 154
    },
    "__intnl_SystemBoolean_26": {
      "name": "__intnl_SystemBoolean_26",
      "type": "System.Boolean",
      "address": 207
    },
    "__1_inkId__param": {
      "name": "__1_inkId__param",
      "type": "System.Int32",
      "address": 40
    },
    "__intnl_VRCSDK3DataDataToken_2": {
      "name": "__intnl_VRCSDK3DataDataToken_2",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 98
    },
    "__gintnl_SystemUInt32_9": {
      "name": "__gintnl_SystemUInt32_9",
      "type": "System.UInt32",
      "address": 76
    },
    "__gintnl_SystemUInt32_8": {
      "name": "__gintnl_SystemUInt32_8",
      "type": "System.UInt32",
      "address": 71
    },
    "__gintnl_SystemUInt32_5": {
      "name": "__gintnl_SystemUInt32_5",
      "type": "System.UInt32",
      "address": 60
    },
    "__gintnl_SystemUInt32_4": {
      "name": "__gintnl_SystemUInt32_4",
      "type": "System.UInt32",
      "address": 54
    },
    "__gintnl_SystemUInt32_7": {
      "name": "__gintnl_SystemUInt32_7",
      "type": "System.UInt32",
      "address": 68
    },
    "__gintnl_SystemUInt32_6": {
      "name": "__gintnl_SystemUInt32_6",
      "type": "System.UInt32",
      "address": 64
    },
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 18
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 16
    },
    "__gintnl_SystemUInt32_3": {
      "name": "__gintnl_SystemUInt32_3",
      "type": "System.UInt32",
      "address": 50
    },
    "__gintnl_SystemUInt32_2": {
      "name": "__gintnl_SystemUInt32_2",
      "type": "System.UInt32",
      "address": 45
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 37
    },
    "__const_SystemBoolean_1": {
      "name": "__const_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 38
    },
    "__0___0_RemoveInk__ret": {
      "name": "__0___0_RemoveInk__ret",
      "type": "System.Boolean",
      "address": 42
    },
    "__0_get_logPrefix__ret": {
      "name": "__0_get_logPrefix__ret",
      "type": "System.String",
      "address": 65
    },
    "__lcl_index_SystemInt32_0": {
      "name": "__lcl_index_SystemInt32_0",
      "type": "System.Int32",
      "address": 144
    },
    "__const_VRCSDKBaseVRC_PickupPickupHand_0": {
      "name": "__const_VRCSDKBaseVRC_PickupPickupHand_0",
      "type": "VRC.SDKBase.VRC_Pickup+PickupHand",
      "address": 23
    },
    "__0___0_RemoveUserInk__ret": {
      "name": "__0___0_RemoveUserInk__ret",
      "type": "System.Boolean",
      "address": 47
    },
    "__const_SystemString_5": {
      "name": "__const_SystemString_5",
      "type": "System.String",
      "address": 29
    },
    "__const_UnityEngineKeyCode_1": {
      "name": "__const_UnityEngineKeyCode_1",
      "type": "UnityEngine.KeyCode",
      "address": 15
    },
    "__lcl_pen_VRCUdonUdonBehaviour_1": {
      "name": "__lcl_pen_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 147
    },
    "__const_VRCSDK3DataTokenType_0": {
      "name": "__const_VRCSDK3DataTokenType_0",
      "type": "VRC.SDK3.Data.TokenType",
      "address": 32
    },
    "__3__intnlparam": {
      "name": "__3__intnlparam",
      "type": "UnityEngine.GameObject",
      "address": 56
    },
    "__intnl_UnityEngineObject_8": {
      "name": "__intnl_UnityEngineObject_8",
      "type": "UnityEngine.Object",
      "address": 228
    },
    "__intnl_UnityEngineObject_7": {
      "name": "__intnl_UnityEngineObject_7",
      "type": "UnityEngine.Object",
      "address": 226
    },
    "__intnl_UnityEngineObject_6": {
      "name": "__intnl_UnityEngineObject_6",
      "type": "UnityEngine.Object",
      "address": 224
    },
    "__intnl_UnityEngineObject_5": {
      "name": "__intnl_UnityEngineObject_5",
      "type": "UnityEngine.Object",
      "address": 222
    },
    "__intnl_UnityEngineObject_4": {
      "name": "__intnl_UnityEngineObject_4",
      "type": "UnityEngine.Object",
      "address": 221
    },
    "__intnl_UnityEngineObject_3": {
      "name": "__intnl_UnityEngineObject_3",
      "type": "UnityEngine.Object",
      "address": 204
    },
    "__intnl_UnityEngineObject_2": {
      "name": "__intnl_UnityEngineObject_2",
      "type": "UnityEngine.Object",
      "address": 203
    },
    "__intnl_UnityEngineObject_1": {
      "name": "__intnl_UnityEngineObject_1",
      "type": "UnityEngine.Object",
      "address": 176
    },
    "__intnl_UnityEngineObject_0": {
      "name": "__intnl_UnityEngineObject_0",
      "type": "UnityEngine.Object",
      "address": 175
    },
    "__intnl_VRCSDK3DataDataToken_1": {
      "name": "__intnl_VRCSDK3DataDataToken_1",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 97
    },
    "callablePenList": {
      "name": "callablePenList",
      "type": "VRC.SDK3.Data.DataList",
      "address": 5
    },
    "__5_penId__param": {
      "name": "__5_penId__param",
      "type": "System.Int32",
      "address": 61
    },
    "__const_SystemString_12": {
      "name": "__const_SystemString_12",
      "type": "System.String",
      "address": 88
    },
    "__const_SystemString_13": {
      "name": "__const_SystemString_13",
      "type": "System.String",
      "address": 94
    },
    "__const_SystemString_10": {
      "name": "__const_SystemString_10",
      "type": "System.String",
      "address": 85
    },
    "__const_SystemString_11": {
      "name": "__const_SystemString_11",
      "type": "System.String",
      "address": 87
    },
    "__intnl_VRCSDKBaseVRCPlayerApi_0": {
      "name": "__intnl_VRCSDKBaseVRCPlayerApi_0",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 111
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__intnl_SystemObject_7": {
      "name": "__intnl_SystemObject_7",
      "type": "System.Object",
      "address": 193
    },
    "__intnl_UnityEngineMesh_1": {
      "name": "__intnl_UnityEngineMesh_1",
      "type": "UnityEngine.Mesh",
      "address": 202
    },
    "__intnl_UnityEngineMesh_0": {
      "name": "__intnl_UnityEngineMesh_0",
      "type": "UnityEngine.Mesh",
      "address": 174
    },
    "__intnl_UnityEngineMesh_2": {
      "name": "__intnl_UnityEngineMesh_2",
      "type": "UnityEngine.Mesh",
      "address": 220
    },
    "__0_GetPen__ret": {
      "name": "__0_GetPen__ret",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 19
    },
    "__intnl_VRCUdonUdonBehaviour_2": {
      "name": "__intnl_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 132
    },
    "__lcl_pickupR_VRCSDK3ComponentsVRCPickup_0": {
      "name": "__lcl_pickupR_VRCSDK3ComponentsVRCPickup_0",
      "type": "VRC.SDK3.Components.VRCPickup",
      "address": 121
    },
    "__intnl_UnityEngineTransform_5": {
      "name": "__intnl_UnityEngineTransform_5",
      "type": "UnityEngine.Transform",
      "address": 241
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 11
    },
    "__intnl_SystemBoolean_11": {
      "name": "__intnl_SystemBoolean_11",
      "type": "System.Boolean",
      "address": 136
    },
    "__intnl_SystemBoolean_21": {
      "name": "__intnl_SystemBoolean_21",
      "type": "System.Boolean",
      "address": 187
    },
    "__intnl_SystemBoolean_31": {
      "name": "__intnl_SystemBoolean_31",
      "type": "System.Boolean",
      "address": 230
    },
    "__intnl_VRCSDK3DataDataToken_9": {
      "name": "__intnl_VRCSDK3DataDataToken_9",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 159
    },
    "__lcl_inkToken_VRCSDK3DataDataToken_2": {
      "name": "__lcl_inkToken_VRCSDK3DataDataToken_2",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 213
    },
    "__lcl_inkToken_VRCSDK3DataDataToken_0": {
      "name": "__lcl_inkToken_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 167
    },
    "__lcl_inkToken_VRCSDK3DataDataToken_1": {
      "name": "__lcl_inkToken_VRCSDK3DataDataToken_1",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 190
    },
    "__const_UnityEngineVector3_0": {
      "name": "__const_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 21
    },
    "__lcl_inkTokens_VRCSDK3DataDataList_0": {
      "name": "__lcl_inkTokens_VRCSDK3DataDataList_0",
      "type": "VRC.SDK3.Data.DataList",
      "address": 209
    },
    "__0_penId__param": {
      "name": "__0_penId__param",
      "type": "System.Int32",
      "address": 9
    },
    "__0__intnlparam": {
      "name": "__0__intnlparam",
      "type": "System.Int32",
      "address": 51
    },
    "__intnl_SystemSingle_2": {
      "name": "__intnl_SystemSingle_2",
      "type": "System.Single",
      "address": 237
    },
    "__7__intnlparam": {
      "name": "__7__intnlparam",
      "type": "System.String",
      "address": 77
    },
    "_logPrefix": {
      "name": "_logPrefix",
      "type": "System.String",
      "address": 8
    },
    "__intnl_SystemObject_2": {
      "name": "__intnl_SystemObject_2",
      "type": "System.Object",
      "address": 134
    },
    "__intnl_VRCSDK3DataDataDictionary_2": {
      "name": "__intnl_VRCSDK3DataDataDictionary_2",
      "type": "VRC.SDK3.Data.DataDictionary",
      "address": 160
    },
    "__intnl_VRCSDK3DataDataDictionary_1": {
      "name": "__intnl_VRCSDK3DataDataDictionary_1",
      "type": "VRC.SDK3.Data.DataDictionary",
      "address": 155
    },
    "__intnl_VRCSDK3DataDataDictionary_0": {
      "name": "__intnl_VRCSDK3DataDataDictionary_0",
      "type": "VRC.SDK3.Data.DataDictionary",
      "address": 100
    },
    "__intnl_VRCUdonUdonBehaviour_1": {
      "name": "__intnl_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 129
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 26
    },
    "__intnl_UnityEngineTransform_2": {
      "name": "__intnl_UnityEngineTransform_2",
      "type": "UnityEngine.Transform",
      "address": 172
    },
    "__const_SystemSingle_2": {
      "name": "__const_SystemSingle_2",
      "type": "System.Single",
      "address": 93
    },
    "__intnl_SystemBoolean_19": {
      "name": "__intnl_SystemBoolean_19",
      "type": "System.Boolean",
      "address": 171
    },
    "__intnl_SystemBoolean_29": {
      "name": "__intnl_SystemBoolean_29",
      "type": "System.Boolean",
      "address": 217
    },
    "__intnl_UnityEngineVector3_0": {
      "name": "__intnl_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 115
    },
    "__lcl_ownerId_SystemInt32_0": {
      "name": "__lcl_ownerId_SystemInt32_0",
      "type": "System.Int32",
      "address": 184
    },
    "__intnl_SystemBoolean_14": {
      "name": "__intnl_SystemBoolean_14",
      "type": "System.Boolean",
      "address": 148
    },
    "__intnl_SystemBoolean_24": {
      "name": "__intnl_SystemBoolean_24",
      "type": "System.Boolean",
      "address": 194
    },
    "__intnl_SystemBoolean_34": {
      "name": "__intnl_SystemBoolean_34",
      "type": "System.Boolean",
      "address": 246
    },
    "__intnl_VRCSDK3DataDataToken_4": {
      "name": "__intnl_VRCSDK3DataDataToken_4",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 101
    },
    "__const_SystemObject_0": {
      "name": "__const_SystemObject_0",
      "type": "System.Object",
      "address": 24
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 223
    },
    "__const_SystemType_0": {
      "name": "__const_SystemType_0",
      "type": "System.Type",
      "address": 46
    },
    "__lcl_x_VRCSDKBaseVRCPlayerApiTrackingData_0": {
      "name": "__lcl_x_VRCSDKBaseVRCPlayerApiTrackingData_0",
      "type": "VRC.SDKBase.VRCPlayerApi+TrackingData",
      "address": 110
    },
    "__intnl_SystemSingle_1": {
      "name": "__intnl_SystemSingle_1",
      "type": "System.Single",
      "address": 233
    },
    "__intnl_SystemObject_1": {
      "name": "__intnl_SystemObject_1",
      "type": "System.Object",
      "address": 131
    },
    "__0_inkId__param": {
      "name": "__0_inkId__param",
      "type": "System.Int32",
      "address": 35
    },
    "__const_SystemString_7": {
      "name": "__const_SystemString_7",
      "type": "System.String",
      "address": 75
    },
    "inkDictMap": {
      "name": "inkDictMap",
      "type": "VRC.SDK3.Data.DataDictionary",
      "address": 4
    },
    "__4__intnlparam": {
      "name": "__4__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 57
    },
    "__0___0_HasInk__ret": {
      "name": "__0___0_HasInk__ret",
      "type": "System.Boolean",
      "address": 33
    },
    "__const_SystemSingle_1": {
      "name": "__const_SystemSingle_1",
      "type": "System.Single",
      "address": 89
    },
    "__const_VRCSDK3DataTokenType_2": {
      "name": "__const_VRCSDK3DataTokenType_2",
      "type": "VRC.SDK3.Data.TokenType",
      "address": 53
    },
    "__lcl_removedInkIdList_VRCSDK3DataDataList_0": {
      "name": "__lcl_removedInkIdList_VRCSDK3DataDataList_0",
      "type": "VRC.SDK3.Data.DataList",
      "address": 183
    },
    "__const_VRCSDKBaseVRCPlayerApiTrackingDataType_0": {
      "name": "__const_VRCSDKBaseVRCPlayerApiTrackingDataType_0",
      "type": "VRC.SDKBase.VRCPlayerApi+TrackingDataType",
      "address": 20
    },
    "__intnl_SystemBoolean_17": {
      "name": "__intnl_SystemBoolean_17",
      "type": "System.Boolean",
      "address": 157
    },
    "__intnl_SystemBoolean_27": {
      "name": "__intnl_SystemBoolean_27",
      "type": "System.Boolean",
      "address": 212
    },
    "__lcl_idHolder_UnityEngineTransform_0": {
      "name": "__lcl_idHolder_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 244
    },
    "__intnl_VRCSDK3DataDataToken_3": {
      "name": "__intnl_VRCSDK3DataDataToken_3",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 99
    },
    "__intnl_SystemInt32_11": {
      "name": "__intnl_SystemInt32_11",
      "type": "System.Int32",
      "address": 252
    },
    "__4_penId__param": {
      "name": "__4_penId__param",
      "type": "System.Int32",
      "address": 48
    },
    "__lcl_inkDict_VRCSDK3DataDataDictionary_1": {
      "name": "__lcl_inkDict_VRCSDK3DataDataDictionary_1",
      "type": "VRC.SDK3.Data.DataDictionary",
      "address": 179
    },
    "__lcl_inkDict_VRCSDK3DataDataDictionary_0": {
      "name": "__lcl_inkDict_VRCSDK3DataDataDictionary_0",
      "type": "VRC.SDK3.Data.DataDictionary",
      "address": 163
    },
    "__lcl_inkDict_VRCSDK3DataDataDictionary_2": {
      "name": "__lcl_inkDict_VRCSDK3DataDataDictionary_2",
      "type": "VRC.SDK3.Data.DataDictionary",
      "address": 208
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "__const_SystemString_4": {
      "name": "__const_SystemString_4",
      "type": "System.String",
      "address": 28
    },
    "__const_UnityEngineKeyCode_0": {
      "name": "__const_UnityEngineKeyCode_0",
      "type": "UnityEngine.KeyCode",
      "address": 14
    },
    "__lcl_pen_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_pen_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 108
    },
    "__const_VRCSDK3DataTokenType_1": {
      "name": "__const_VRCSDK3DataTokenType_1",
      "type": "VRC.SDK3.Data.TokenType",
      "address": 36
    },
    "__lcl_i_SystemInt32_1": {
      "name": "__lcl_i_SystemInt32_1",
      "type": "System.Int32",
      "address": 185
    },
    "__lcl_i_SystemInt32_0": {
      "name": "__lcl_i_SystemInt32_0",
      "type": "System.Int32",
      "address": 141
    },
    "__lcl_i_SystemInt32_2": {
      "name": "__lcl_i_SystemInt32_2",
      "type": "System.Int32",
      "address": 210
    },
    "__3_penId__param": {
      "name": "__3_penId__param",
      "type": "System.Int32",
      "address": 43
    },
    "__intnl_SystemSingle_4": {
      "name": "__intnl_SystemSingle_4",
      "type": "System.Single",
      "address": 249
    },
    "__1__intnlparam": {
      "name": "__1__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 52
    },
    "__lcl_pickupL_VRCSDK3ComponentsVRCPickup_0": {
      "name": "__lcl_pickupL_VRCSDK3ComponentsVRCPickup_0",
      "type": "VRC.SDK3.Components.VRCPickup",
      "address": 124
    },
    "__intnl_SystemObject_4": {
      "name": "__intnl_SystemObject_4",
      "type": "System.Object",
      "address": 149
    },
    "__lcl_penToken_VRCSDK3DataDataToken_0": {
      "name": "__lcl_penToken_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 145
    },
    "__intnl_VRCUdonUdonBehaviour_3": {
      "name": "__intnl_VRCUdonUdonBehaviour_3",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 139
    },
    "__intnl_UnityEngineTransform_4": {
      "name": "__intnl_UnityEngineTransform_4",
      "type": "UnityEngine.Transform",
      "address": 218
    },
    "__const_SystemString_9": {
      "name": "__const_SystemString_9",
      "type": "System.String",
      "address": 83
    },
    "__intnl_UnityEngineVector3_2": {
      "name": "__intnl_UnityEngineVector3_2",
      "type": "UnityEngine.Vector3",
      "address": 117
    },
    "__intnl_SystemBoolean_12": {
      "name": "__intnl_SystemBoolean_12",
      "type": "System.Boolean",
      "address": 143
    },
    "__intnl_SystemBoolean_22": {
      "name": "__intnl_SystemBoolean_22",
      "type": "System.Boolean",
      "address": 189
    },
    "__intnl_SystemBoolean_32": {
      "name": "__intnl_SystemBoolean_32",
      "type": "System.Boolean",
      "address": 240
    },
    "__intnl_VRCSDK3DataDataToken_6": {
      "name": "__intnl_VRCSDK3DataDataToken_6",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 152
    },
    "__const_UnityEngineVector3_1": {
      "name": "__const_UnityEngineVector3_1",
      "type": "UnityEngine.Vector3",
      "address": 92
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "__gintnl_SystemObjectArray_0": {
      "name": "__gintnl_SystemObjectArray_0",
      "type": "System.Object[]",
      "address": 79
    },
    "__0_inkInstance__param": {
      "name": "__0_inkInstance__param",
      "type": "UnityEngine.GameObject",
      "address": 41
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 138
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 135
    },
    "__intnl_SystemInt32_3": {
      "name": "__intnl_SystemInt32_3",
      "type": "System.Int32",
      "address": 234
    },
    "__intnl_SystemInt32_2": {
      "name": "__intnl_SystemInt32_2",
      "type": "System.Int32",
      "address": 232
    },
    "__intnl_SystemInt32_5": {
      "name": "__intnl_SystemInt32_5",
      "type": "System.Int32",
      "address": 236
    },
    "__intnl_SystemInt32_4": {
      "name": "__intnl_SystemInt32_4",
      "type": "System.Int32",
      "address": 235
    },
    "__intnl_SystemInt32_7": {
      "name": "__intnl_SystemInt32_7",
      "type": "System.Int32",
      "address": 239
    },
    "__intnl_SystemInt32_6": {
      "name": "__intnl_SystemInt32_6",
      "type": "System.Int32",
      "address": 238
    },
    "__intnl_SystemInt32_9": {
      "name": "__intnl_SystemInt32_9",
      "type": "System.Int32",
      "address": 248
    },
    "__intnl_SystemInt32_8": {
      "name": "__intnl_SystemInt32_8",
      "type": "System.Int32",
      "address": 242
    },
    "__intnl_SystemString_2": {
      "name": "__intnl_SystemString_2",
      "type": "System.String",
      "address": 227
    },
    "__lcl_inkOwnerIdVector_UnityEngineVector3_0": {
      "name": "__lcl_inkOwnerIdVector_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 197
    },
    "__1_o__param": {
      "name": "__1_o__param",
      "type": "System.Object",
      "address": 67
    },
    "__0_o__param": {
      "name": "__0_o__param",
      "type": "System.Object",
      "address": 62
    },
    "__0_c__param": {
      "name": "__0_c__param",
      "type": "UnityEngine.Color",
      "address": 74
    },
    "__intnl_SystemSingle_3": {
      "name": "__intnl_SystemSingle_3",
      "type": "System.Single",
      "address": 247
    },
    "__2_o__param": {
      "name": "__2_o__param",
      "type": "System.Object",
      "address": 70
    },
    "__intnl_UnityEngineMeshCollider_0": {
      "name": "__intnl_UnityEngineMeshCollider_0",
      "type": "UnityEngine.MeshCollider",
      "address": 173
    },
    "__intnl_UnityEngineMeshCollider_1": {
      "name": "__intnl_UnityEngineMeshCollider_1",
      "type": "UnityEngine.MeshCollider",
      "address": 201
    },
    "__intnl_UnityEngineMeshCollider_2": {
      "name": "__intnl_UnityEngineMeshCollider_2",
      "type": "UnityEngine.MeshCollider",
      "address": 219
    },
    "__intnl_SystemObject_3": {
      "name": "__intnl_SystemObject_3",
      "type": "System.Object",
      "address": 140
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 13
    },
    "__intnl_UnityEngineTransform_1": {
      "name": "__intnl_UnityEngineTransform_1",
      "type": "UnityEngine.Transform",
      "address": 118
    },
    "__5__intnlparam": {
      "name": "__5__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 58
    },
    "__intnl_UnityEngineVector3_1": {
      "name": "__intnl_UnityEngineVector3_1",
      "type": "UnityEngine.Vector3",
      "address": 116
    },
    "__intnl_VRCSDK3DataDataToken_18": {
      "name": "__intnl_VRCSDK3DataDataToken_18",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 205
    },
    "__intnl_VRCSDK3DataDataToken_14": {
      "name": "__intnl_VRCSDK3DataDataToken_14",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 166
    },
    "__intnl_VRCSDK3DataDataToken_15": {
      "name": "__intnl_VRCSDK3DataDataToken_15",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 177
    },
    "__intnl_VRCSDK3DataDataToken_16": {
      "name": "__intnl_VRCSDK3DataDataToken_16",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 180
    },
    "__intnl_VRCSDK3DataDataToken_17": {
      "name": "__intnl_VRCSDK3DataDataToken_17",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 181
    },
    "__intnl_VRCSDK3DataDataToken_10": {
      "name": "__intnl_VRCSDK3DataDataToken_10",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 161
    },
    "__intnl_VRCSDK3DataDataToken_11": {
      "name": "__intnl_VRCSDK3DataDataToken_11",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 162
    },
    "__intnl_VRCSDK3DataDataToken_12": {
      "name": "__intnl_VRCSDK3DataDataToken_12",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 164
    },
    "__intnl_VRCSDK3DataDataToken_13": {
      "name": "__intnl_VRCSDK3DataDataToken_13",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 165
    },
    "__lcl__discard1_UnityEngineVector3_0": {
      "name": "__lcl__discard1_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 195
    },
    "__intnl_SystemBoolean_15": {
      "name": "__intnl_SystemBoolean_15",
      "type": "System.Boolean",
      "address": 150
    },
    "__intnl_SystemBoolean_25": {
      "name": "__intnl_SystemBoolean_25",
      "type": "System.Boolean",
      "address": 199
    },
    "__intnl_VRCSDK3DataDataToken_5": {
      "name": "__intnl_VRCSDK3DataDataToken_5",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 105
    },
    "penDict": {
      "name": "penDict",
      "type": "VRC.SDK3.Data.DataDictionary",
      "address": 3
    },
    "__lcl_n_SystemInt32_2": {
      "name": "__lcl_n_SystemInt32_2",
      "type": "System.Int32",
      "address": 211
    },
    "__lcl_n_SystemInt32_0": {
      "name": "__lcl_n_SystemInt32_0",
      "type": "System.Int32",
      "address": 142
    },
    "__lcl_n_SystemInt32_1": {
      "name": "__lcl_n_SystemInt32_1",
      "type": "System.Int32",
      "address": 186
    },
    "__const_VRCSDKBaseVRC_PickupPickupHand_1": {
      "name": "__const_VRCSDKBaseVRC_PickupPickupHand_1",
      "type": "VRC.SDKBase.VRC_Pickup+PickupHand",
      "address": 25
    },
    "__const_SystemString_6": {
      "name": "__const_SystemString_6",
      "type": "System.String",
      "address": 63
    },
    "logColor": {
      "name": "logColor",
      "type": "UnityEngine.Color",
      "address": 7
    },
    "__intnl_UnityEngineVector3_4": {
      "name": "__intnl_UnityEngineVector3_4",
      "type": "UnityEngine.Vector3",
      "address": 120
    },
    "__gintnl_SystemUInt32_10": {
      "name": "__gintnl_SystemUInt32_10",
      "type": "System.UInt32",
      "address": 80
    },
    "__intnl_VRCSDK3DataDataToken_0": {
      "name": "__intnl_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 95
    },
    "__intnl_SystemInt32_10": {
      "name": "__intnl_SystemInt32_10",
      "type": "System.Int32",
      "address": 250
    },
    "__2_penId__param": {
      "name": "__2_penId__param",
      "type": "System.Int32",
      "address": 39
    },
    "__intnl_VRCSDKBaseVRCPlayerApi_1": {
      "name": "__intnl_VRCSDKBaseVRCPlayerApi_1",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 122
    },
    "__intnl_SystemObject_6": {
      "name": "__intnl_SystemObject_6",
      "type": "System.Object",
      "address": 170
    },
    "__lcl_inkIdList_VRCSDK3DataDataList_0": {
      "name": "__lcl_inkIdList_VRCSDK3DataDataList_0",
      "type": "VRC.SDK3.Data.DataList",
      "address": 182
    },
    "__intnl_UnityEngineTransform_6": {
      "name": "__intnl_UnityEngineTransform_6",
      "type": "UnityEngine.Transform",
      "address": 245
    },
    "__2__intnlparam": {
      "name": "__2__intnlparam",
      "type": "System.Boolean",
      "address": 55
    },
    "__intnl_SystemBoolean_10": {
      "name": "__intnl_SystemBoolean_10",
      "type": "System.Boolean",
      "address": 133
    },
    "__intnl_SystemBoolean_20": {
      "name": "__intnl_SystemBoolean_20",
      "type": "System.Boolean",
      "address": 178
    },
    "__intnl_SystemBoolean_30": {
      "name": "__intnl_SystemBoolean_30",
      "type": "System.Boolean",
      "address": 229
    },
    "__intnl_UnityEngineQuaternion_0": {
      "name": "__intnl_UnityEngineQuaternion_0",
      "type": "UnityEngine.Quaternion",
      "address": 113
    },
    "__intnl_VRCSDK3DataDataToken_8": {
      "name": "__intnl_VRCSDK3DataDataToken_8",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 158
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 31
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 30
    },
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 84
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 82
    },
    "__const_SystemInt32_5": {
      "name": "__const_SystemInt32_5",
      "type": "System.Int32",
      "address": 90
    },
    "__const_SystemInt32_4": {
      "name": "__const_SystemInt32_4",
      "type": "System.Int32",
      "address": 86
    },
    "__const_SystemInt32_6": {
      "name": "__const_SystemInt32_6",
      "type": "System.Int32",
      "address": 91
    },
    "__1_pen__param": {
      "name": "__1_pen__param",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 17
    },
    "__intnl_SystemBoolean_8": {
      "name": "__intnl_SystemBoolean_8",
      "type": "System.Boolean",
      "address": 128
    },
    "__intnl_SystemBoolean_9": {
      "name": "__intnl_SystemBoolean_9",
      "type": "System.Boolean",
      "address": 130
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 96
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 103
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 106
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 107
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 109
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 123
    },
    "__intnl_SystemBoolean_6": {
      "name": "__intnl_SystemBoolean_6",
      "type": "System.Boolean",
      "address": 126
    },
    "__intnl_SystemBoolean_7": {
      "name": "__intnl_SystemBoolean_7",
      "type": "System.Boolean",
      "address": 127
    },
    "__0_pen__param": {
      "name": "__0_pen__param",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 10
    },
    "__intnl_SystemSingle_5": {
      "name": "__intnl_SystemSingle_5",
      "type": "System.Single",
      "address": 251
    },
    "__intnl_VRCSDKBaseVRCPlayerApi_2": {
      "name": "__intnl_VRCSDKBaseVRCPlayerApi_2",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 125
    },
    "__intnl_SystemObject_5": {
      "name": "__intnl_SystemObject_5",
      "type": "System.Object",
      "address": 151
    },
    "__intnl_SystemObject_8": {
      "name": "__intnl_SystemObject_8",
      "type": "System.Object",
      "address": 216
    },
    "__intnl_VRCUdonUdonBehaviour_0": {
      "name": "__intnl_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 102
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 27
    },
    "__this_VRCUdonUdonBehaviour_2": {
      "name": "__this_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 72
    },
    "__this_VRCUdonUdonBehaviour_1": {
      "name": "__this_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 69
    },
    "__this_VRCUdonUdonBehaviour_0": {
      "name": "__this_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 66
    },
    "__intnl_UnityEngineTransform_3": {
      "name": "__intnl_UnityEngineTransform_3",
      "type": "UnityEngine.Transform",
      "address": 200
    },
    "__const_SystemString_8": {
      "name": "__const_SystemString_8",
      "type": "System.String",
      "address": 81
    },
    "__intnl_SystemBoolean_18": {
      "name": "__intnl_SystemBoolean_18",
      "type": "System.Boolean",
      "address": 168
    },
    "__intnl_SystemBoolean_28": {
      "name": "__intnl_SystemBoolean_28",
      "type": "System.Boolean",
      "address": 214
    },
    "__intnl_UnityEngineVector3_3": {
      "name": "__intnl_UnityEngineVector3_3",
      "type": "UnityEngine.Vector3",
      "address": 119
    },
    "__lcl_inkDictToken_VRCSDK3DataDataToken_1": {
      "name": "__lcl_inkDictToken_VRCSDK3DataDataToken_1",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 206
    },
    "__lcl_inkDictToken_VRCSDK3DataDataToken_0": {
      "name": "__lcl_inkDictToken_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 153
    },
    "__2_inkId__param": {
      "name": "__2_inkId__param",
      "type": "System.Int32",
      "address": 44
    }
  },
  "entryPoints": [
    {
      "name": "__0_Register",
      "address": 0
    },
    {
      "name": "_update",
      "address": 456
    },
    {
      "name": "__0_SetLastUsedPen",
      "address": 640
    },
    {
      "name": "__0_HasInk",
      "address": 2816
    },
    {
      "name": "__0_SetInk",
      "address": 3164
    },
    {
      "name": "__0_RemoveInk",
      "address": 3352
    },
    {
      "name": "__0_RemoveUserInk",
      "address": 4160
    },
    {
      "name": "__0_Clear",
      "address": 5548
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 8320864560273335438
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.QvPen_Manager"
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
      "type": "VRC.SDK3.Data.DataDictionary",
      "value": {
        "isSerializable": true,
        "value": []
      }
    },
    "4": {
      "address": 4,
      "type": "VRC.SDK3.Data.DataDictionary",
      "value": {
        "isSerializable": true,
        "value": []
      }
    },
    "5": {
      "address": 5,
      "type": "VRC.SDK3.Data.DataList",
      "value": {
        "isSerializable": true,
        "value": []
      }
    },
    "6": {
      "address": 6,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "7": {
      "address": 7,
      "type": "UnityEngine.Color",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Color",
          "toString": "RGBA(0.949, 0.490, 0.290, 1.000)"
        }
      }
    },
    "8": {
      "address": 8,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "9": {
      "address": 9,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "10": {
      "address": 10,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "11": {
      "address": 11,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "12": {
      "address": 12,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "get_AllowCallPen"
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_get_AllowCallPen__ret"
      }
    },
    "14": {
      "address": 14,
      "type": "UnityEngine.KeyCode",
      "value": {
        "isSerializable": true,
        "value": 113
      }
    },
    "15": {
      "address": 15,
      "type": "UnityEngine.KeyCode",
      "value": {
        "isSerializable": true,
        "value": 9
      }
    },
    "16": {
      "address": 16,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 620
      }
    },
    "17": {
      "address": 17,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "18": {
      "address": 18,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 712
      }
    },
    "19": {
      "address": 19,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "20": {
      "address": 20,
      "type": "VRC.SDKBase.VRCPlayerApi+TrackingDataType",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "21": {
      "address": 21,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 1.00)"
        }
      }
    },
    "22": {
      "address": 22,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.5
      }
    },
    "23": {
      "address": 23,
      "type": "VRC.SDKBase.VRC_Pickup+PickupHand",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "24": {
      "address": 24,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "25": {
      "address": 25,
      "type": "VRC.SDKBase.VRC_Pickup+PickupHand",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "26": {
      "address": 26,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "get_isHeld"
      }
    },
    "27": {
      "address": 27,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_get_isHeld__ret"
      }
    },
    "28": {
      "address": 28,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_TakeOwnership"
      }
    },
    "29": {
      "address": 29,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__TakeOwnership__ret"
      }
    },
    "30": {
      "address": 30,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "31": {
      "address": 31,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "32": {
      "address": 32,
      "type": "VRC.SDK3.Data.TokenType",
      "value": {
        "isSerializable": true,
        "value": 15
      }
    },
    "33": {
      "address": 33,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "34": {
      "address": 34,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "35": {
      "address": 35,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "36": {
      "address": 36,
      "type": "VRC.SDK3.Data.TokenType",
      "value": {
        "isSerializable": true,
        "value": 14
      }
    },
    "37": {
      "address": 37,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "38": {
      "address": 38,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "39": {
      "address": 39,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "40": {
      "address": 40,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "41": {
      "address": 41,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "42": {
      "address": 42,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "43": {
      "address": 43,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "44": {
      "address": 44,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "45": {
      "address": 45,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3416
      }
    },
    "46": {
      "address": 46,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "49": {
      "address": 49,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "50": {
      "address": 50,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4324
      }
    },
    "51": {
      "address": 51,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "52": {
      "address": 52,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "53": {
      "address": 53,
      "type": "VRC.SDK3.Data.TokenType",
      "value": {
        "isSerializable": true,
        "value": 6
      }
    },
    "54": {
      "address": 54,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4816
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
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "57": {
      "address": 57,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "58": {
      "address": 58,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "59": {
      "address": 59,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "60": {
      "address": 60,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4944
      }
    },
    "61": {
      "address": 61,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "62": {
      "address": 62,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "63": {
      "address": 63,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "{0}{1}"
      }
    },
    "64": {
      "address": 64,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6252
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
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "67": {
      "address": 67,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "68": {
      "address": 68,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6380
      }
    },
    "69": {
      "address": 69,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "70": {
      "address": 70,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "71": {
      "address": 71,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6508
      }
    },
    "72": {
      "address": 72,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
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
      "type": "UnityEngine.Color",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Color",
          "toString": "RGBA(0.000, 0.000, 0.000, 0.000)"
        }
      }
    },
    "75": {
      "address": 75,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "<color=\"#{0}\">"
      }
    },
    "76": {
      "address": 76,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6656
      }
    },
    "77": {
      "address": 77,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "78": {
      "address": 78,
      "type": "UnityEngine.Color",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Color",
          "toString": "RGBA(0.000, 0.000, 0.000, 0.000)"
        }
      }
    },
    "79": {
      "address": 79,
      "type": "System.Object[]",
      "value": {
        "isSerializable": true,
        "value": [
          null,
          null,
          null,
          null,
          null
        ]
      }
    },
    "80": {
      "address": 80,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6844
      }
    },
    "81": {
      "address": 81,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen"
      }
    },
    "82": {
      "address": 82,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "83": {
      "address": 83,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Udon"
      }
    },
    "84": {
      "address": 84,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "85": {
      "address": 85,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen_Manager"
      }
    },
    "86": {
      "address": 86,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 4
      }
    },
    "87": {
      "address": 87,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "</color>"
      }
    },
    "88": {
      "address": 88,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "[{0}{1}.{2}.{3}{4}] "
      }
    },
    "89": {
      "address": 89,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 4.0
      }
    },
    "90": {
      "address": 90,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 360
      }
    },
    "91": {
      "address": 91,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 129600
      }
    },
    "92": {
      "address": 92,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "93": {
      "address": 93,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 255.0
      }
    },
    "94": {
      "address": 94,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "{0:x2}{1:x2}{2:x2}"
      }
    },
    "95": {
      "address": 95,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "96": {
      "address": 96,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "97": {
      "address": 97,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "98": {
      "address": 98,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "99": {
      "address": 99,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "100": {
      "address": 100,
      "type": "VRC.SDK3.Data.DataDictionary",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "101": {
      "address": 101,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "102": {
      "address": 102,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "103": {
      "address": 103,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "104": {
      "address": 104,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "105": {
      "address": 105,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "106": {
      "address": 106,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "109": {
      "address": 109,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "110": {
      "address": 110,
      "type": "VRC.SDKBase.VRCPlayerApi+TrackingData",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDKBase.VRCPlayerApi+TrackingData",
          "toString": "VRC.SDKBase.VRCPlayerApi+TrackingData"
        }
      }
    },
    "111": {
      "address": 111,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "112": {
      "address": 112,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "113": {
      "address": 113,
      "type": "UnityEngine.Quaternion",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Quaternion",
          "toString": "(0.00000, 0.00000, 0.00000, 0.00000)"
        }
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
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "116": {
      "address": 116,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "117": {
      "address": 117,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "118": {
      "address": 118,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "119": {
      "address": 119,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "120": {
      "address": 120,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "121": {
      "address": 121,
      "type": "VRC.SDK3.Components.VRCPickup",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "122": {
      "address": 122,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "VRC.SDK3.Components.VRCPickup",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "127": {
      "address": 127,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "132": {
      "address": 132,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "135": {
      "address": 135,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "136": {
      "address": 136,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "137": {
      "address": 137,
      "type": "System.Int32[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "138": {
      "address": 138,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "139": {
      "address": 139,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "140": {
      "address": 140,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "141": {
      "address": 141,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "142": {
      "address": 142,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "145": {
      "address": 145,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "146": {
      "address": 146,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "149": {
      "address": 149,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "152": {
      "address": 152,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "153": {
      "address": 153,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "154": {
      "address": 154,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "155": {
      "address": 155,
      "type": "VRC.SDK3.Data.DataDictionary",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "156": {
      "address": 156,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "157": {
      "address": 157,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "158": {
      "address": 158,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "159": {
      "address": 159,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "160": {
      "address": 160,
      "type": "VRC.SDK3.Data.DataDictionary",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "161": {
      "address": 161,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
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
      "type": "VRC.SDK3.Data.DataDictionary",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "164": {
      "address": 164,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "165": {
      "address": 165,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "166": {
      "address": 166,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
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
      "type": "UnityEngine.GameObject",
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
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "173": {
      "address": 173,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "174": {
      "address": 174,
      "type": "UnityEngine.Mesh",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "175": {
      "address": 175,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "176": {
      "address": 176,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "177": {
      "address": 177,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "178": {
      "address": 178,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "179": {
      "address": 179,
      "type": "VRC.SDK3.Data.DataDictionary",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "180": {
      "address": 180,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "181": {
      "address": 181,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "182": {
      "address": 182,
      "type": "VRC.SDK3.Data.DataList",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "183": {
      "address": 183,
      "type": "VRC.SDK3.Data.DataList",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "184": {
      "address": 184,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "185": {
      "address": 185,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "189": {
      "address": 189,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "190": {
      "address": 190,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "191": {
      "address": 191,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "192": {
      "address": 192,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "193": {
      "address": 193,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "194": {
      "address": 194,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "195": {
      "address": 195,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "196": {
      "address": 196,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "197": {
      "address": 197,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "198": {
      "address": 198,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "199": {
      "address": 199,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "200": {
      "address": 200,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "201": {
      "address": 201,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "202": {
      "address": 202,
      "type": "UnityEngine.Mesh",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "203": {
      "address": 203,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "204": {
      "address": 204,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "205": {
      "address": 205,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "206": {
      "address": 206,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "207": {
      "address": 207,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "208": {
      "address": 208,
      "type": "VRC.SDK3.Data.DataDictionary",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "209": {
      "address": 209,
      "type": "VRC.SDK3.Data.DataList",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "210": {
      "address": 210,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "211": {
      "address": 211,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "212": {
      "address": 212,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "213": {
      "address": 213,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "214": {
      "address": 214,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "215": {
      "address": 215,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "216": {
      "address": 216,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "217": {
      "address": 217,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "218": {
      "address": 218,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "219": {
      "address": 219,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "220": {
      "address": 220,
      "type": "UnityEngine.Mesh",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "221": {
      "address": 221,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "222": {
      "address": 222,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "223": {
      "address": 223,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "224": {
      "address": 224,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "225": {
      "address": 225,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "226": {
      "address": 226,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "227": {
      "address": 227,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "228": {
      "address": 228,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "229": {
      "address": 229,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "230": {
      "address": 230,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "231": {
      "address": 231,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "232": {
      "address": 232,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "233": {
      "address": 233,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "234": {
      "address": 234,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "235": {
      "address": 235,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "236": {
      "address": 236,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "237": {
      "address": 237,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "238": {
      "address": 238,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "239": {
      "address": 239,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "240": {
      "address": 240,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "241": {
      "address": 241,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "242": {
      "address": 242,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "243": {
      "address": 243,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "244": {
      "address": 244,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "245": {
      "address": 245,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "246": {
      "address": 246,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "247": {
      "address": 247,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "248": {
      "address": 248,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "249": {
      "address": 249,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "250": {
      "address": 250,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "251": {
      "address": 251,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "252": {
      "address": 252,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "253": {
      "address": 253,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__op_Implicit__SystemInt32__VRCSDK3DataDataToken"
      }
    },
    "254": {
      "address": 254,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__ContainsKey__VRCSDK3DataDataToken__SystemBoolean"
      }
    },
    "255": {
      "address": 255,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__op_Implicit__UnityEngineObject__VRCSDK3DataDataToken"
      }
    },
    "256": {
      "address": 256,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__set_Item__VRCSDK3DataDataToken_VRCSDK3DataDataToken__SystemVoid"
      }
    },
    "257": {
      "address": 257,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__ctor____VRCSDK3DataDataDictionary"
      }
    },
    "258": {
      "address": 258,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__op_Implicit__VRCSDK3DataDataDictionary__VRCSDK3DataDataToken"
      }
    },
    "259": {
      "address": 259,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEvent__SystemString__SystemVoid"
      }
    },
    "260": {
      "address": 260,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariable__SystemString__SystemObject"
      }
    },
    "261": {
      "address": 261,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__Add__VRCSDK3DataDataToken__SystemVoid"
      }
    },
    "262": {
      "address": 262,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineInput.__get_anyKeyDown__SystemBoolean"
      }
    },
    "263": {
      "address": 263,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineInput.__GetKeyDown__UnityEngineKeyCode__SystemBoolean"
      }
    },
    "264": {
      "address": 264,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineInput.__GetKey__UnityEngineKeyCode__SystemBoolean"
      }
    },
    "265": {
      "address": 265,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseUtilities.__IsValid__SystemObject__SystemBoolean"
      }
    },
    "266": {
      "address": 266,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__get_LocalPlayer__VRCSDKBaseVRCPlayerApi"
      }
    },
    "267": {
      "address": 267,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__GetTrackingData__VRCSDKBaseVRCPlayerApiTrackingDataType__VRCSDKBaseVRCPlayerApiTrackingData"
      }
    },
    "268": {
      "address": 268,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApiTrackingData.__get_rotation__UnityEngineQuaternion"
      }
    },
    "269": {
      "address": 269,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineQuaternion.__op_Multiply__UnityEngineQuaternion_UnityEngineVector3__UnityEngineVector3"
      }
    },
    "270": {
      "address": 270,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__get_transform__UnityEngineTransform"
      }
    },
    "271": {
      "address": 271,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApiTrackingData.__get_position__UnityEngineVector3"
      }
    },
    "272": {
      "address": 272,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Multiply__SystemSingle_UnityEngineVector3__UnityEngineVector3"
      }
    },
    "273": {
      "address": 273,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Addition__UnityEngineVector3_UnityEngineVector3__UnityEngineVector3"
      }
    },
    "274": {
      "address": 274,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__set_position__UnityEngineVector3__SystemVoid"
      }
    },
    "275": {
      "address": 275,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__LookAt__UnityEngineVector3__SystemVoid"
      }
    },
    "276": {
      "address": 276,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__GetPickupInHand__VRCSDKBaseVRC_PickupPickupHand__VRCSDKBaseVRC_Pickup"
      }
    },
    "277": {
      "address": 277,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_UnaryNegation__SystemBoolean__SystemBoolean"
      }
    },
    "278": {
      "address": 278,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__get_Count__SystemInt32"
      }
    },
    "279": {
      "address": 279,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "280": {
      "address": 280,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32Array.__ctor__SystemInt32__SystemInt32Array"
      }
    },
    "281": {
      "address": 281,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "282": {
      "address": 282,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32Array.__Set__SystemInt32_SystemInt32__SystemVoid"
      }
    },
    "283": {
      "address": 283,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "284": {
      "address": 284,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseUtilities.__ShuffleArray__SystemInt32Array__SystemVoid"
      }
    },
    "285": {
      "address": 285,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__get_Length__SystemInt32"
      }
    },
    "286": {
      "address": 286,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32Array.__Get__SystemInt32__SystemInt32"
      }
    },
    "287": {
      "address": 287,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__TryGetValue__SystemInt32_VRCSDK3DataTokenType_VRCSDK3DataDataTokenRef__SystemBoolean"
      }
    },
    "288": {
      "address": 288,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__get_Reference__SystemObject"
      }
    },
    "289": {
      "address": 289,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__TryGetValue__VRCSDK3DataDataToken_VRCSDK3DataTokenType_VRCSDK3DataDataTokenRef__SystemBoolean"
      }
    },
    "290": {
      "address": 290,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__get_DataDictionary__VRCSDK3DataDataDictionary"
      }
    },
    "291": {
      "address": 291,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__get_Item__VRCSDK3DataDataToken__VRCSDK3DataDataToken"
      }
    },
    "292": {
      "address": 292,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__Remove__VRCSDK3DataDataToken__SystemBoolean"
      }
    },
    "293": {
      "address": 293,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__get_transform__UnityEngineTransform"
      }
    },
    "294": {
      "address": 294,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__GetComponentInChildren__SystemBoolean__T"
      }
    },
    "295": {
      "address": 295,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMeshCollider.__get_sharedMesh__UnityEngineMesh"
      }
    },
    "296": {
      "address": 296,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__Destroy__UnityEngineObject__SystemVoid"
      }
    },
    "297": {
      "address": 297,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__GetKeys__VRCSDK3DataDataList"
      }
    },
    "298": {
      "address": 298,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__ctor____VRCSDK3DataDataList"
      }
    },
    "299": {
      "address": 299,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Inequality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "300": {
      "address": 300,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__GetValues__VRCSDK3DataDataList"
      }
    },
    "301": {
      "address": 301,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__Clear__SystemVoid"
      }
    },
    "302": {
      "address": 302,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject_SystemObject__SystemString"
      }
    },
    "303": {
      "address": 303,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject_UnityEngineObject__SystemVoid"
      }
    },
    "304": {
      "address": 304,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__LogWarning__SystemObject_UnityEngineObject__SystemVoid"
      }
    },
    "305": {
      "address": 305,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__LogError__SystemObject_UnityEngineObject__SystemVoid"
      }
    },
    "306": {
      "address": 306,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject__SystemString"
      }
    },
    "307": {
      "address": 307,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__IsNullOrEmpty__SystemString__SystemBoolean"
      }
    },
    "308": {
      "address": 308,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Set__SystemInt32_SystemObject__SystemVoid"
      }
    },
    "309": {
      "address": 309,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObjectArray__SystemString"
      }
    },
    "310": {
      "address": 310,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Multiply__UnityEngineVector3_SystemSingle__UnityEngineVector3"
      }
    },
    "311": {
      "address": 311,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__get_x__SystemSingle"
      }
    },
    "312": {
      "address": 312,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__RoundToInt__SystemSingle__SystemInt32"
      }
    },
    "313": {
      "address": 313,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__get_y__SystemSingle"
      }
    },
    "314": {
      "address": 314,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Multiplication__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "315": {
      "address": 315,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__get_z__SystemSingle"
      }
    },
    "316": {
      "address": 316,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_childCount__SystemInt32"
      }
    },
    "317": {
      "address": 317,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__GetChild__SystemInt32__UnityEngineTransform"
      }
    },
    "318": {
      "address": 318,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_localPosition__UnityEngineVector3"
      }
    },
    "319": {
      "address": 319,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_localScale__UnityEngineVector3"
      }
    },
    "320": {
      "address": 320,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_localEulerAngles__UnityEngineVector3"
      }
    },
    "321": {
      "address": 321,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__op_Multiply__UnityEngineColor_SystemSingle__UnityEngineColor"
      }
    },
    "322": {
      "address": 322,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__get_r__SystemSingle"
      }
    },
    "323": {
      "address": 323,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__get_g__SystemSingle"
      }
    },
    "324": {
      "address": 324,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__get_b__SystemSingle"
      }
    },
    "325": {
      "address": 325,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject_SystemObject_SystemObject__SystemString"
      }
    }
  }
}
```