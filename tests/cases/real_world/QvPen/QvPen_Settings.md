<!-- ci: skip-compile -->

```csharp
﻿using TMPro;
using UdonSharp;
using UnityEngine;
using UnityEngine.UI;
using Utilities = VRC.SDKBase.Utilities;

#pragma warning disable IDE0044
#pragma warning disable IDE0090, IDE1006

namespace QvPen.UdonScript
{
    [AddComponentMenu("")]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_Settings : UdonSharpBehaviour
    {
        [System.NonSerialized]
        public string version = string.Empty;

        [SerializeField]
        private TextAsset versionText;

        [SerializeField]
        private Text information;
        [SerializeField]
        private TextMeshPro informationTMP;
        [SerializeField]
        private TextMeshProUGUI informationTMPU;

        [SerializeField]
        private Transform pensParent;
        [SerializeField]
        private Transform erasersParent;

        [System.NonSerialized]
        public QvPen_PenManager[] penManagers = { };

        [System.NonSerialized]
        public QvPen_EraserManager[] eraserManagers = { };

        private void Start()
        {
            if (Utilities.IsValid(versionText))
                version = versionText.text.Trim();

#if !UNITY_EDITOR
            const string ureishi = nameof(ureishi);
            Log($"{nameof(QvPen)} {version} - {ureishi}");
#endif

            var informationText =
                    $"<size=20></size>\n" +
                    $"<size=14>{version}</size>";

            if (Utilities.IsValid(information))
                information.text = informationText;
            if (Utilities.IsValid(informationTMP))
                informationTMP.text = informationText;
            if (Utilities.IsValid(informationTMPU))
                informationTMPU.text = informationText;

            if (Utilities.IsValid(pensParent))
                penManagers = pensParent.GetComponentsInChildren<QvPen_PenManager>();
            if (Utilities.IsValid(erasersParent))
                eraserManagers = erasersParent.GetComponentsInChildren<QvPen_EraserManager>();
        }

        #region Log

        private const string appName = nameof(QvPen_Settings);

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
  "byteCodeHex": "000000010000000E00000001000000040000000100000041000000060000007E0000000100000041000000040000006000000001000000040000000100000042000000060000007F0000000100000042000000010000000300000006000000800000000100000010000000010000000300000001000000440000000600000081000000010000000F00000001000000440000000100000043000000060000008200000001000000050000000100000045000000060000007E000000010000004500000004000000E000000001000000050000000100000043000000060000008300000001000000060000000100000046000000060000007E0000000100000046000000040000012000000001000000060000000100000043000000060000008400000001000000070000000100000047000000060000007E0000000100000047000000040000016000000001000000070000000100000043000000060000008500000001000000080000000100000048000000060000007E000000010000004800000004000001D80000000100000011000000010000000800000001000000490000000600000086000000010000004900000001000000130000000900000005000005B40000000100000012000000010000000A000000090000000100000009000000010000004A000000060000007E000000010000004A000000040000025000000001000000140000000100000009000000010000004B0000000600000086000000010000004B00000001000000160000000900000005000006480000000100000015000000010000000B000000090000000100000002000000090000000800000002000000010000000E0000000100000019000000050000044C0000000100000018000000010000001A0000000100000017000000010000004C0000000600000087000000010000001B000000010000004D00000009000000010000004C000000010000004D00000006000000880000000100000002000000090000000800000002000000010000000E000000010000001D000000050000044C0000000100000018000000010000001A000000010000001C000000010000004E0000000600000087000000010000001E000000010000004F00000009000000010000004E000000010000004F00000006000000890000000100000002000000090000000800000002000000010000000E0000000100000020000000050000044C0000000100000018000000010000001A000000010000001F00000001000000500000000600000087000000010000002100000001000000510000000900000001000000500000000100000051000000060000008A0000000100000002000000090000000800000002000000010000000E0000000100000025000000010000002300000001000000270000000900000005000006DC00000001000000240000000100000026000000010000002200000006000000810000000100000002000000090000000800000002000000010000000E000000010000000D0000000100000052000000060000008B00000001000000520000000100000053000000060000008C000000010000005300000004000004A8000000010000000D000000010000001A0000000900000005000005A0000000010000002A000000010000000C00000001000000230000000900000005000003EC000000010000002800000001000000290000000100000022000000060000008D0000000100000028000000010000002B000000010000002C000000060000008D0000000100000028000000010000002D000000010000002E000000060000008D0000000100000028000000010000002F0000000100000030000000060000008D000000010000002800000001000000310000000100000032000000060000008D00000001000000330000000100000028000000010000000D000000060000008E000000010000000D000000010000001A000000090000000100000002000000090000000800000002000000010000001300000001000000340000000100000055000000060000008F00000001000000550000000100000054000000090000000100000035000000010000005400000001000000370000000900000005000007E4000000010000003600000001000000120000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000001600000001000000340000000100000057000000060000008F000000010000005700000001000000560000000900000001000000380000000100000056000000010000003A000000090000000500000C4C0000000100000039000000010000001500000009000000010000000200000009000000080000000200000001000000020000000900000008000000020000000100000027000000010000003B000000010000002700000006000000900000000100000027000000010000005800000006000000910000000100000058000000010000005900000006000000920000000100000027000000010000005A0000000600000093000000010000005A000000010000005B00000006000000920000000100000027000000010000005C0000000600000094000000010000005C000000010000005D0000000600000092000000010000003C0000000100000059000000010000005B000000010000005D0000000100000026000000060000009500000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000003D000000010000005E000000090000000100000029000000010000005F0000000900000001000000370000000100000062000000060000009600000001000000290000000100000063000000090000000100000063000000010000006200000001000000640000000600000097000000010000006400000004000009D400000001000000370000000100000063000000010000006500000006000000980000000100000065000000010000003E00000001000000660000000600000099000000010000003F000000010000006700000009000000010000006600000001000000670000000100000068000000060000009A000000010000006800000004000008F400000005000009AC0000000100000065000000010000003E0000000100000069000000060000009B0000000100000069000000010000003F000000010000006A000000060000009C000000010000006A000000040000097C0000000100000069000000010000006B000000060000009D000000010000006B000000010000005E000000010000006A000000060000009E000000010000006A00000004000009AC000000010000005F000000010000002B000000010000005F000000060000009F0000000100000063000000010000002B0000000100000063000000060000009F0000000500000838000000010000005F000000010000006000000006000000A000000001000000290000000100000061000000090000000100000037000000010000006200000006000000960000000100000029000000010000006300000009000000010000006300000001000000620000000100000064000000060000009700000001000000640000000400000C1000000001000000370000000100000063000000010000006500000006000000980000000100000065000000010000003E00000001000000660000000600000099000000010000003F000000010000006700000009000000010000006600000001000000670000000100000068000000060000009A00000001000000680000000400000AE80000000500000BE80000000100000065000000010000003E0000000100000069000000060000009B0000000100000069000000010000003F000000010000006A000000060000009C000000010000006A0000000400000B700000000100000069000000010000006B000000060000009D000000010000006B000000010000005E000000010000006A000000060000009E000000010000006A0000000400000BE80000000100000061000000010000006C00000009000000010000006C000000010000002B0000000100000061000000060000009F0000000100000065000000010000006D000000090000000100000060000000010000006C000000010000006D000000060000008D0000000100000063000000010000002B0000000100000063000000060000009F0000000500000A2C0000000100000060000000010000003600000009000000010000000200000009000000080000000200000001000000020000000900000008000000020000000100000040000000010000006E000000090000000100000029000000010000006F00000009000000010000003A000000010000007200000006000000960000000100000029000000010000007300000009000000010000007300000001000000720000000100000074000000060000009700000001000000740000000400000E3C000000010000003A0000000100000073000000010000007500000006000000980000000100000075000000010000003E00000001000000760000000600000099000000010000003F000000010000007700000009000000010000007600000001000000770000000100000078000000060000009A00000001000000780000000400000D5C0000000500000E140000000100000075000000010000003E0000000100000079000000060000009B0000000100000079000000010000003F000000010000007A000000060000009C000000010000007A0000000400000DE40000000100000079000000010000007B000000060000009D000000010000007B000000010000006E000000010000007A000000060000009E000000010000007A0000000400000E14000000010000006F000000010000002B000000010000006F000000060000009F0000000100000073000000010000002B0000000100000073000000060000009F0000000500000CA0000000010000006F000000010000007000000006000000A00000000100000029000000010000007100000009000000010000003A000000010000007200000006000000960000000100000029000000010000007300000009000000010000007300000001000000720000000100000074000000060000009700000001000000740000000400001078000000010000003A0000000100000073000000010000007500000006000000980000000100000075000000010000003E00000001000000760000000600000099000000010000003F000000010000007700000009000000010000007600000001000000770000000100000078000000060000009A00000001000000780000000400000F5000000005000010500000000100000075000000010000003E0000000100000079000000060000009B0000000100000079000000010000003F000000010000007A000000060000009C000000010000007A0000000400000FD80000000100000079000000010000007B000000060000009D000000010000007B000000010000006E000000010000007A000000060000009E000000010000007A00000004000010500000000100000071000000010000007C00000009000000010000007C000000010000002B0000000100000071000000060000009F0000000100000075000000010000007D000000090000000100000070000000010000007C000000010000007D000000060000008D0000000100000073000000010000002B0000000100000073000000060000009F0000000500000E94000000010000007000000001000000390000000900000001000000020000000900000008000000020000000100000002000000090000000800000002",
  "byteCodeLength": 4276,
  "symbols": {
    "__intnl_SystemBoolean_13": {
      "name": "__intnl_SystemBoolean_13",
      "type": "System.Boolean",
      "address": 122
    },
    "__lcl_instanceBehaviours_UnityEngineComponentArray_1": {
      "name": "__lcl_instanceBehaviours_UnityEngineComponentArray_1",
      "type": "UnityEngine.Component[]",
      "address": 86
    },
    "__const_SystemInt64_1": {
      "name": "__const_SystemInt64_1",
      "type": "System.Int64",
      "address": 64
    },
    "__const_SystemInt64_0": {
      "name": "__const_SystemInt64_0",
      "type": "System.Int64",
      "address": 61
    },
    "__lcl_targetIdx_SystemInt32_0": {
      "name": "__lcl_targetIdx_SystemInt32_0",
      "type": "System.Int32",
      "address": 97
    },
    "__lcl_targetIdx_SystemInt32_1": {
      "name": "__lcl_targetIdx_SystemInt32_1",
      "type": "System.Int32",
      "address": 113
    },
    "__intnl_SystemString_1": {
      "name": "__intnl_SystemString_1",
      "type": "System.String",
      "address": 68
    },
    "__6__intnlparam": {
      "name": "__6__intnlparam",
      "type": "UnityEngine.Component[]",
      "address": 54
    },
    "__8__intnlparam": {
      "name": "__8__intnlparam",
      "type": "UnityEngine.Component[]",
      "address": 57
    },
    "__intnl_SystemSingle_0": {
      "name": "__intnl_SystemSingle_0",
      "type": "System.Single",
      "address": 88
    },
    "eraserManagers": {
      "name": "eraserManagers",
      "type": "UnityEngine.Component[]",
      "address": 11
    },
    "__lcl_targetID_SystemInt64_1": {
      "name": "__lcl_targetID_SystemInt64_1",
      "type": "System.Int64",
      "address": 110
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 15
    },
    "__intnl_UnityEngineTransform_0": {
      "name": "__intnl_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 73
    },
    "__const_SystemSingle_0": {
      "name": "__const_SystemSingle_0",
      "type": "System.Single",
      "address": 59
    },
    "__0___0_ColorBeginTag__ret": {
      "name": "__0___0_ColorBeginTag__ret",
      "type": "System.String",
      "address": 34
    },
    "__intnl_SystemType_3": {
      "name": "__intnl_SystemType_3",
      "type": "System.Type",
      "address": 119
    },
    "__gintnl_SystemUInt32_8": {
      "name": "__gintnl_SystemUInt32_8",
      "type": "System.UInt32",
      "address": 56
    },
    "__gintnl_SystemUInt32_5": {
      "name": "__gintnl_SystemUInt32_5",
      "type": "System.UInt32",
      "address": 37
    },
    "__gintnl_SystemUInt32_4": {
      "name": "__gintnl_SystemUInt32_4",
      "type": "System.UInt32",
      "address": 32
    },
    "__gintnl_SystemUInt32_7": {
      "name": "__gintnl_SystemUInt32_7",
      "type": "System.UInt32",
      "address": 53
    },
    "__gintnl_SystemUInt32_6": {
      "name": "__gintnl_SystemUInt32_6",
      "type": "System.UInt32",
      "address": 42
    },
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 20
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 17
    },
    "__gintnl_SystemUInt32_3": {
      "name": "__gintnl_SystemUInt32_3",
      "type": "System.UInt32",
      "address": 29
    },
    "__gintnl_SystemUInt32_2": {
      "name": "__gintnl_SystemUInt32_2",
      "type": "System.UInt32",
      "address": 25
    },
    "__0_get_logPrefix__ret": {
      "name": "__0_get_logPrefix__ret",
      "type": "System.String",
      "address": 26
    },
    "__lcl_foundBehaviours_UnityEngineComponentArray_0": {
      "name": "__lcl_foundBehaviours_UnityEngineComponentArray_0",
      "type": "UnityEngine.Component[]",
      "address": 96
    },
    "__lcl_foundBehaviours_UnityEngineComponentArray_1": {
      "name": "__lcl_foundBehaviours_UnityEngineComponentArray_1",
      "type": "UnityEngine.Component[]",
      "address": 112
    },
    "__const_SystemString_5": {
      "name": "__const_SystemString_5",
      "type": "System.String",
      "address": 46
    },
    "__3__intnlparam": {
      "name": "__3__intnlparam",
      "type": "UnityEngine.Component",
      "address": 22
    },
    "__intnl_UnityEngineObject_2": {
      "name": "__intnl_UnityEngineObject_2",
      "type": "UnityEngine.Object",
      "address": 81
    },
    "__intnl_UnityEngineObject_1": {
      "name": "__intnl_UnityEngineObject_1",
      "type": "UnityEngine.Object",
      "address": 79
    },
    "__intnl_UnityEngineObject_0": {
      "name": "__intnl_UnityEngineObject_0",
      "type": "UnityEngine.Object",
      "address": 77
    },
    "__const_SystemString_10": {
      "name": "__const_SystemString_10",
      "type": "System.String",
      "address": 62
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "versionText": {
      "name": "versionText",
      "type": "UnityEngine.TextAsset",
      "address": 4
    },
    "penManagers": {
      "name": "penManagers",
      "type": "UnityEngine.Component[]",
      "address": 10
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 14
    },
    "__intnl_UnityEngineComponent_0": {
      "name": "__intnl_UnityEngineComponent_0",
      "type": "UnityEngine.Component",
      "address": 109
    },
    "__intnl_SystemBoolean_11": {
      "name": "__intnl_SystemBoolean_11",
      "type": "System.Boolean",
      "address": 116
    },
    "__lcl_informationText_SystemString_0": {
      "name": "__lcl_informationText_SystemString_0",
      "type": "System.String",
      "address": 67
    },
    "__intnl_SystemString_3": {
      "name": "__intnl_SystemString_3",
      "type": "System.String",
      "address": 78
    },
    "__0__intnlparam": {
      "name": "__0__intnlparam",
      "type": "UnityEngine.Component[]",
      "address": 18
    },
    "__intnl_SystemSingle_2": {
      "name": "__intnl_SystemSingle_2",
      "type": "System.Single",
      "address": 92
    },
    "__7__intnlparam": {
      "name": "__7__intnlparam",
      "type": "UnityEngine.Component[]",
      "address": 55
    },
    "_logPrefix": {
      "name": "_logPrefix",
      "type": "System.String",
      "address": 13
    },
    "__9__intnlparam": {
      "name": "__9__intnlparam",
      "type": "UnityEngine.Component[]",
      "address": 58
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 24
    },
    "__intnl_UnityEngineComponentArray_1": {
      "name": "__intnl_UnityEngineComponentArray_1",
      "type": "UnityEngine.Component[]",
      "address": 87
    },
    "__intnl_UnityEngineComponentArray_0": {
      "name": "__intnl_UnityEngineComponentArray_0",
      "type": "UnityEngine.Component[]",
      "address": 85
    },
    "__const_SystemObject_0": {
      "name": "__const_SystemObject_0",
      "type": "System.Object",
      "address": 63
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 66
    },
    "__intnl_SystemType_1": {
      "name": "__intnl_SystemType_1",
      "type": "System.Type",
      "address": 103
    },
    "__const_SystemType_0": {
      "name": "__const_SystemType_0",
      "type": "System.Type",
      "address": 52
    },
    "__intnl_SystemSingle_1": {
      "name": "__intnl_SystemSingle_1",
      "type": "System.Single",
      "address": 90
    },
    "__const_SystemString_7": {
      "name": "__const_SystemString_7",
      "type": "System.String",
      "address": 50
    },
    "__4__intnlparam": {
      "name": "__4__intnlparam",
      "type": "System.String",
      "address": 38
    },
    "__intnl_SystemType_2": {
      "name": "__intnl_SystemType_2",
      "type": "System.Type",
      "address": 118
    },
    "version": {
      "name": "version",
      "type": "System.String",
      "address": 3
    },
    "__intnl_SystemInt64_1": {
      "name": "__intnl_SystemInt64_1",
      "type": "System.Int64",
      "address": 123
    },
    "__intnl_SystemInt64_0": {
      "name": "__intnl_SystemInt64_0",
      "type": "System.Int64",
      "address": 107
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "__const_SystemString_4": {
      "name": "__const_SystemString_4",
      "type": "System.String",
      "address": 44
    },
    "__lcl_behaviour_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_behaviour_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 101
    },
    "__1__intnlparam": {
      "name": "__1__intnlparam",
      "type": "UnityEngine.Component",
      "address": 19
    },
    "__const_SystemString_9": {
      "name": "__const_SystemString_9",
      "type": "System.String",
      "address": 60
    },
    "__intnl_SystemBoolean_12": {
      "name": "__intnl_SystemBoolean_12",
      "type": "System.Boolean",
      "address": 120
    },
    "__lcl_instanceBehaviours_UnityEngineComponentArray_0": {
      "name": "__lcl_instanceBehaviours_UnityEngineComponentArray_0",
      "type": "UnityEngine.Component[]",
      "address": 84
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "__gintnl_SystemObjectArray_0": {
      "name": "__gintnl_SystemObjectArray_0",
      "type": "System.Object[]",
      "address": 40
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 91
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 89
    },
    "__intnl_SystemInt32_3": {
      "name": "__intnl_SystemInt32_3",
      "type": "System.Int32",
      "address": 98
    },
    "__intnl_SystemInt32_2": {
      "name": "__intnl_SystemInt32_2",
      "type": "System.Int32",
      "address": 93
    },
    "__intnl_SystemInt32_5": {
      "name": "__intnl_SystemInt32_5",
      "type": "System.Int32",
      "address": 108
    },
    "__intnl_SystemInt32_4": {
      "name": "__intnl_SystemInt32_4",
      "type": "System.Int32",
      "address": 99
    },
    "__intnl_SystemInt32_7": {
      "name": "__intnl_SystemInt32_7",
      "type": "System.Int32",
      "address": 115
    },
    "__intnl_SystemInt32_6": {
      "name": "__intnl_SystemInt32_6",
      "type": "System.Int32",
      "address": 114
    },
    "__intnl_SystemInt32_8": {
      "name": "__intnl_SystemInt32_8",
      "type": "System.Int32",
      "address": 124
    },
    "__intnl_SystemString_2": {
      "name": "__intnl_SystemString_2",
      "type": "System.String",
      "address": 76
    },
    "__1_o__param": {
      "name": "__1_o__param",
      "type": "System.Object",
      "address": 28
    },
    "__0_o__param": {
      "name": "__0_o__param",
      "type": "System.Object",
      "address": 23
    },
    "__0_c__param": {
      "name": "__0_c__param",
      "type": "UnityEngine.Color",
      "address": 35
    },
    "__2_o__param": {
      "name": "__2_o__param",
      "type": "System.Object",
      "address": 31
    },
    "informationTMP": {
      "name": "informationTMP",
      "type": "TMPro.TextMeshPro",
      "address": 6
    },
    "__lcl_targetID_SystemInt64_0": {
      "name": "__lcl_targetID_SystemInt64_0",
      "type": "System.Int64",
      "address": 94
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 16
    },
    "__intnl_UnityEngineTransform_1": {
      "name": "__intnl_UnityEngineTransform_1",
      "type": "UnityEngine.Transform",
      "address": 75
    },
    "__5__intnlparam": {
      "name": "__5__intnlparam",
      "type": "UnityEngine.Color",
      "address": 39
    },
    "information": {
      "name": "information",
      "type": "UnityEngine.UI.Text",
      "address": 5
    },
    "pensParent": {
      "name": "pensParent",
      "type": "UnityEngine.Transform",
      "address": 8
    },
    "__intnl_SystemType_0": {
      "name": "__intnl_SystemType_0",
      "type": "System.Type",
      "address": 102
    },
    "__lcl_arraySize_SystemInt32_0": {
      "name": "__lcl_arraySize_SystemInt32_0",
      "type": "System.Int32",
      "address": 95
    },
    "__lcl_arraySize_SystemInt32_1": {
      "name": "__lcl_arraySize_SystemInt32_1",
      "type": "System.Int32",
      "address": 111
    },
    "__const_SystemString_6": {
      "name": "__const_SystemString_6",
      "type": "System.String",
      "address": 48
    },
    "logColor": {
      "name": "logColor",
      "type": "UnityEngine.Color",
      "address": 12
    },
    "informationTMPU": {
      "name": "informationTMPU",
      "type": "TMPro.TextMeshProUGUI",
      "address": 7
    },
    "erasersParent": {
      "name": "erasersParent",
      "type": "UnityEngine.Transform",
      "address": 9
    },
    "__2__intnlparam": {
      "name": "__2__intnlparam",
      "type": "UnityEngine.Component[]",
      "address": 21
    },
    "__intnl_UnityEngineComponent_1": {
      "name": "__intnl_UnityEngineComponent_1",
      "type": "UnityEngine.Component",
      "address": 125
    },
    "__intnl_SystemBoolean_10": {
      "name": "__intnl_SystemBoolean_10",
      "type": "System.Boolean",
      "address": 106
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 43
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 41
    },
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 47
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 45
    },
    "__const_SystemInt32_4": {
      "name": "__const_SystemInt32_4",
      "type": "System.Int32",
      "address": 49
    },
    "__lcl_behaviour_VRCUdonUdonBehaviour_1": {
      "name": "__lcl_behaviour_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 117
    },
    "__intnl_SystemBoolean_8": {
      "name": "__intnl_SystemBoolean_8",
      "type": "System.Boolean",
      "address": 100
    },
    "__intnl_SystemBoolean_9": {
      "name": "__intnl_SystemBoolean_9",
      "type": "System.Boolean",
      "address": 104
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 65
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 69
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 70
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 71
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 72
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 74
    },
    "__intnl_SystemBoolean_6": {
      "name": "__intnl_SystemBoolean_6",
      "type": "System.Boolean",
      "address": 82
    },
    "__intnl_SystemBoolean_7": {
      "name": "__intnl_SystemBoolean_7",
      "type": "System.Boolean",
      "address": 83
    },
    "__intnl_SystemString_4": {
      "name": "__intnl_SystemString_4",
      "type": "System.String",
      "address": 80
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 36
    },
    "__this_VRCUdonUdonBehaviour_2": {
      "name": "__this_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 33
    },
    "__this_VRCUdonUdonBehaviour_1": {
      "name": "__this_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 30
    },
    "__this_VRCUdonUdonBehaviour_0": {
      "name": "__this_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 27
    },
    "__const_SystemString_8": {
      "name": "__const_SystemString_8",
      "type": "System.String",
      "address": 51
    },
    "__lcl_typeID_SystemObject_1": {
      "name": "__lcl_typeID_SystemObject_1",
      "type": "System.Object",
      "address": 121
    },
    "__lcl_typeID_SystemObject_0": {
      "name": "__lcl_typeID_SystemObject_0",
      "type": "System.Object",
      "address": 105
    }
  },
  "entryPoints": [
    {
      "name": "_start",
      "address": 0
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": -5575584688282760284
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.QvPen_Settings"
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": ""
      }
    },
    "4": {
      "address": 4,
      "type": "UnityEngine.TextAsset",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "5": {
      "address": 5,
      "type": "UnityEngine.UI.Text",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "6": {
      "address": 6,
      "type": "TMPro.TextMeshPro",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "7": {
      "address": 7,
      "type": "TMPro.TextMeshProUGUI",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "8": {
      "address": 8,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "9": {
      "address": 9,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "10": {
      "address": 10,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": []
      }
    },
    "11": {
      "address": 11,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": []
      }
    },
    "12": {
      "address": 12,
      "type": "UnityEngine.Color",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Color",
          "toString": "RGBA(0.949, 0.490, 0.290, 1.000)"
        }
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "14": {
      "address": 14,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "15": {
      "address": 15,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "<size=20></size>\n"
      }
    },
    "16": {
      "address": 16,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "<size=14>{0}</size>"
      }
    },
    "17": {
      "address": 17,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 452
      }
    },
    "18": {
      "address": 18,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "19": {
      "address": 19,
      "type": "UnityEngine.Component",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "20": {
      "address": 20,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 572
      }
    },
    "21": {
      "address": 21,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "22": {
      "address": 22,
      "type": "UnityEngine.Component",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "23": {
      "address": 23,
      "type": "System.Object",
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
        "value": "{0}{1}"
      }
    },
    "25": {
      "address": 25,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 636
      }
    },
    "26": {
      "address": 26,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "27": {
      "address": 27,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
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
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 764
      }
    },
    "30": {
      "address": 30,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "31": {
      "address": 31,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "32": {
      "address": 32,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 892
      }
    },
    "33": {
      "address": 33,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "34": {
      "address": 34,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "35": {
      "address": 35,
      "type": "UnityEngine.Color",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Color",
          "toString": "RGBA(0.000, 0.000, 0.000, 0.000)"
        }
      }
    },
    "36": {
      "address": 36,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "<color=\"#{0}\">"
      }
    },
    "37": {
      "address": 37,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1040
      }
    },
    "38": {
      "address": 38,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "39": {
      "address": 39,
      "type": "UnityEngine.Color",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Color",
          "toString": "RGBA(0.000, 0.000, 0.000, 0.000)"
        }
      }
    },
    "40": {
      "address": 40,
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
    "41": {
      "address": 41,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "42": {
      "address": 42,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1228
      }
    },
    "43": {
      "address": 43,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "44": {
      "address": 44,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen"
      }
    },
    "45": {
      "address": 45,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "46": {
      "address": 46,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Udon"
      }
    },
    "47": {
      "address": 47,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "48": {
      "address": 48,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen_Settings"
      }
    },
    "49": {
      "address": 49,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 4
      }
    },
    "50": {
      "address": 50,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "</color>"
      }
    },
    "51": {
      "address": 51,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "[{0}{1}.{2}.{3}{4}] "
      }
    },
    "52": {
      "address": 52,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "53": {
      "address": 53,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1548
      }
    },
    "54": {
      "address": 54,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "55": {
      "address": 55,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "56": {
      "address": 56,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1696
      }
    },
    "57": {
      "address": 57,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "58": {
      "address": 58,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "59": {
      "address": 59,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 255.0
      }
    },
    "60": {
      "address": 60,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "{0:x2}{1:x2}{2:x2}"
      }
    },
    "61": {
      "address": 61,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": -1598075768554875822
      }
    },
    "62": {
      "address": 62,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__refl_typeid"
      }
    },
    "63": {
      "address": 63,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "64": {
      "address": 64,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": -5465713458096539185
      }
    },
    "65": {
      "address": 65,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "66": {
      "address": 66,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "67": {
      "address": 67,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "68": {
      "address": 68,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "UnityEngine.Transform",
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
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "76": {
      "address": 76,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "77": {
      "address": 77,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "78": {
      "address": 78,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "79": {
      "address": 79,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "80": {
      "address": 80,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "81": {
      "address": 81,
      "type": "UnityEngine.Object",
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
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "85": {
      "address": 85,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "86": {
      "address": 86,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "87": {
      "address": 87,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "88": {
      "address": 88,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "89": {
      "address": 89,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "90": {
      "address": 90,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "91": {
      "address": 91,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "92": {
      "address": 92,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "93": {
      "address": 93,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "94": {
      "address": 94,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "95": {
      "address": 95,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "96": {
      "address": 96,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "99": {
      "address": 99,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "100": {
      "address": 100,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "101": {
      "address": 101,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "102": {
      "address": 102,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "103": {
      "address": 103,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "104": {
      "address": 104,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "105": {
      "address": 105,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "108": {
      "address": 108,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "109": {
      "address": 109,
      "type": "UnityEngine.Component",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "110": {
      "address": 110,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "111": {
      "address": 111,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "112": {
      "address": 112,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "113": {
      "address": 113,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "114": {
      "address": 114,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "115": {
      "address": 115,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "118": {
      "address": 118,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "119": {
      "address": 119,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "120": {
      "address": 120,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "121": {
      "address": 121,
      "type": "System.Object",
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
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "124": {
      "address": 124,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "125": {
      "address": 125,
      "type": "UnityEngine.Component",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "126": {
      "address": 126,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseUtilities.__IsValid__SystemObject__SystemBoolean"
      }
    },
    "127": {
      "address": 127,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTextAsset.__get_text__SystemString"
      }
    },
    "128": {
      "address": 128,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Trim__SystemString"
      }
    },
    "129": {
      "address": 129,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject__SystemString"
      }
    },
    "130": {
      "address": 130,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__op_Addition__SystemString_SystemString__SystemString"
      }
    },
    "131": {
      "address": 131,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineUIText.__set_text__SystemString__SystemVoid"
      }
    },
    "132": {
      "address": 132,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "TMProTextMeshPro.__set_text__SystemString__SystemVoid"
      }
    },
    "133": {
      "address": 133,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "TMProTextMeshProUGUI.__set_text__SystemString__SystemVoid"
      }
    },
    "134": {
      "address": 134,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__get_transform__UnityEngineTransform"
      }
    },
    "135": {
      "address": 135,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject_SystemObject__SystemString"
      }
    },
    "136": {
      "address": 136,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject_UnityEngineObject__SystemVoid"
      }
    },
    "137": {
      "address": 137,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__LogWarning__SystemObject_UnityEngineObject__SystemVoid"
      }
    },
    "138": {
      "address": 138,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__LogError__SystemObject_UnityEngineObject__SystemVoid"
      }
    },
    "139": {
      "address": 139,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__IsNullOrEmpty__SystemString__SystemBoolean"
      }
    },
    "140": {
      "address": 140,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_UnaryNegation__SystemBoolean__SystemBoolean"
      }
    },
    "141": {
      "address": 141,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Set__SystemInt32_SystemObject__SystemVoid"
      }
    },
    "142": {
      "address": 142,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObjectArray__SystemString"
      }
    },
    "143": {
      "address": 143,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__GetComponentsInChildren__SystemType__UnityEngineComponentArray"
      }
    },
    "144": {
      "address": 144,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__op_Multiply__UnityEngineColor_SystemSingle__UnityEngineColor"
      }
    },
    "145": {
      "address": 145,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__get_r__SystemSingle"
      }
    },
    "146": {
      "address": 146,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__RoundToInt__SystemSingle__SystemInt32"
      }
    },
    "147": {
      "address": 147,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__get_g__SystemSingle"
      }
    },
    "148": {
      "address": 148,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__get_b__SystemSingle"
      }
    },
    "149": {
      "address": 149,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject_SystemObject_SystemObject__SystemString"
      }
    },
    "150": {
      "address": 150,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__get_Length__SystemInt32"
      }
    },
    "151": {
      "address": 151,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "152": {
      "address": 152,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Get__SystemInt32__SystemObject"
      }
    },
    "153": {
      "address": 153,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariableType__SystemString__SystemType"
      }
    },
    "154": {
      "address": 154,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemType.__op_Equality__SystemType_SystemType__SystemBoolean"
      }
    },
    "155": {
      "address": 155,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariable__SystemString__SystemObject"
      }
    },
    "156": {
      "address": 156,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObject.__op_Inequality__SystemObject_SystemObject__SystemBoolean"
      }
    },
    "157": {
      "address": 157,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemConvert.__ToInt64__SystemObject__SystemInt64"
      }
    },
    "158": {
      "address": 158,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt64.__op_Equality__SystemInt64_SystemInt64__SystemBoolean"
      }
    },
    "159": {
      "address": 159,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "160": {
      "address": 160,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponentArray.__ctor__SystemInt32__UnityEngineComponentArray"
      }
    }
  }
}
```