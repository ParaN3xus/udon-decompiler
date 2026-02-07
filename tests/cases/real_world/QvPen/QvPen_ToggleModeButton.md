<!-- ci: skip-compile -->

```csharp
using UdonSharp;
using UnityEngine;
using Utilities = VRC.SDKBase.Utilities;

#pragma warning disable IDE0044

namespace QvPen.UdonScript.UI
{
    enum QvPen_ToggleModeButton_Mode
    {
        [InspectorName("Nop")]
        Nop,
        [InspectorName("Use Double Click")]
        UseDoubleClick,
        [InspectorName("Enabled Late Sync")]
        EnabledSync,
        [InspectorName("Use Surftrace Mode")]
        UseSurftraceMode
    }

    [AddComponentMenu("")]
    [DefaultExecutionOrder(30)]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_ToggleModeButton : UdonSharpBehaviour
    {
        [SerializeField]
        private QvPen_Settings settings;

        [SerializeField]
        private QvPen_ToggleModeButton_Mode mode = QvPen_ToggleModeButton_Mode.Nop;

        [SerializeField]
        private bool isOn = false;

        [SerializeField]
        private GameObject displayObjectOn;
        [SerializeField]
        private GameObject displayObjectOff;

        private void Start() => UpdateEnabled();

        public override void Interact()
        {
            isOn ^= true;
            UpdateEnabled();
        }

        private void UpdateEnabled()
        {
            if (Utilities.IsValid(displayObjectOn))
                displayObjectOn.SetActive(isOn);
            if (Utilities.IsValid(displayObjectOff))
                displayObjectOff.SetActive(!isOn);

            switch (mode)
            {
                case QvPen_ToggleModeButton_Mode.UseDoubleClick:
                    foreach (var penManager in settings.penManagers)
                    {
                        if (Utilities.IsValid(penManager))
                            penManager._SetUsingDoubleClick(isOn);
                    }
                    break;
                case QvPen_ToggleModeButton_Mode.EnabledSync:
                    foreach (var penManager in settings.penManagers)
                    {
                        if (Utilities.IsValid(penManager))
                            penManager._SetEnabledLateSync(isOn);
                    }
                    break;
                case QvPen_ToggleModeButton_Mode.UseSurftraceMode:
                    foreach (var penManager in settings.penManagers)
                    {
                        if (Utilities.IsValid(penManager))
                            penManager._SetUsingSurftraceMode(isOn);
                    }
                    break;
            }
        }
    }
}
```

```json
{
  "byteCodeHex": "000000010000000800000001000000090000000500000080000000010000000200000009000000080000000200000001000000080000000100000005000000010000000A00000001000000050000000600000026000000010000000B000000050000008000000001000000020000000900000008000000020000000100000008000000010000000600000001000000180000000600000027000000010000001800000004000000C0000000010000000600000001000000050000000600000028000000010000000700000001000000190000000600000027000000010000001900000004000001180000000100000005000000010000001A00000006000000290000000100000007000000010000001A00000006000000280000000100000004000000010000000C000000010000001B000000060000002A000000010000001B000000040000029C0000000100000003000000010000000D000000010000001E000000060000002B000000010000001E000000010000001F00000009000000010000001F0000000100000020000000060000002C000000010000000E00000001000000210000000900000001000000210000000100000020000000010000001C000000060000002D000000010000001C0000000400000294000000010000001F00000001000000210000000100000022000000060000002E0000000100000022000000010000001D0000000600000027000000010000001D000000040000026C00000001000000050000000100000023000000090000000100000022000000010000000F0000000100000005000000060000002F000000010000002200000001000000100000000600000030000000010000002100000001000000110000000100000021000000060000003100000005000001A800000005000005A400000001000000040000000100000012000000010000001C000000060000002A000000010000001C00000004000004200000000100000003000000010000000D000000010000001E000000060000002B000000010000001E000000010000001F00000009000000010000001F0000000100000020000000060000002C000000010000000E00000001000000210000000900000001000000210000000100000020000000010000001D000000060000002D000000010000001D0000000400000418000000010000001F00000001000000210000000100000022000000060000002E000000010000002200000001000000230000000600000027000000010000002300000004000003F00000000100000005000000010000002400000009000000010000002200000001000000130000000100000005000000060000002F0000000100000022000000010000001400000006000000300000000100000021000000010000001100000001000000210000000600000031000000050000032C00000005000005A400000001000000040000000100000015000000010000001D000000060000002A000000010000001D00000004000005A40000000100000003000000010000000D000000010000001E000000060000002B000000010000001E000000010000001F00000009000000010000001F0000000100000020000000060000002C000000010000000E000000010000002100000009000000010000002100000001000000200000000100000023000000060000002D0000000100000023000000040000059C000000010000001F00000001000000210000000100000022000000060000002E000000010000002200000001000000240000000600000027000000010000002400000004000005740000000100000005000000010000002500000009000000010000002200000001000000160000000100000005000000060000002F000000010000002200000001000000170000000600000030000000010000002100000001000000110000000100000021000000060000003100000005000004B000000005000005A40000000100000002000000090000000800000002",
  "byteCodeLength": 1464,
  "symbols": {
    "__intnl_SystemObject_0": {
      "name": "__intnl_SystemObject_0",
      "type": "System.Object",
      "address": 30
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 13
    },
    "isOn": {
      "name": "isOn",
      "type": "System.Boolean",
      "address": 5
    },
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 11
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 9
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 10
    },
    "__lcl_penManager_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_penManager_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 34
    },
    "__const_SystemString_5": {
      "name": "__const_SystemString_5",
      "type": "System.String",
      "address": 22
    },
    "displayObjectOff": {
      "name": "displayObjectOff",
      "type": "UnityEngine.GameObject",
      "address": 7
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 8
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 16
    },
    "__intnl_UnityEngineComponentArray_0": {
      "name": "__intnl_UnityEngineComponentArray_0",
      "type": "UnityEngine.Component[]",
      "address": 31
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "__const_SystemString_4": {
      "name": "__const_SystemString_4",
      "type": "System.String",
      "address": 20
    },
    "displayObjectOn": {
      "name": "displayObjectOn",
      "type": "UnityEngine.GameObject",
      "address": 6
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "settings": {
      "name": "settings",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 3
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 33
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 32
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 15
    },
    "__const_SystemString_6": {
      "name": "__const_SystemString_6",
      "type": "System.String",
      "address": 23
    },
    "mode": {
      "name": "mode",
      "type": "System.Int32",
      "address": 4
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 14
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 12
    },
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 18
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 17
    },
    "__const_SystemInt32_4": {
      "name": "__const_SystemInt32_4",
      "type": "System.Int32",
      "address": 21
    },
    "__intnl_SystemBoolean_8": {
      "name": "__intnl_SystemBoolean_8",
      "type": "System.Boolean",
      "address": 37
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 24
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 25
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 26
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 27
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 28
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 29
    },
    "__intnl_SystemBoolean_6": {
      "name": "__intnl_SystemBoolean_6",
      "type": "System.Boolean",
      "address": 35
    },
    "__intnl_SystemBoolean_7": {
      "name": "__intnl_SystemBoolean_7",
      "type": "System.Boolean",
      "address": 36
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 19
    }
  },
  "entryPoints": [
    {
      "name": "_start",
      "address": 0
    },
    {
      "name": "_interact",
      "address": 44
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 2136754545500517282
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.UI.QvPen_ToggleModeButton"
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
        "value": 0
      }
    },
    "5": {
      "address": 5,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "8": {
      "address": 8,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "9": {
      "address": 9,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24
      }
    },
    "10": {
      "address": 10,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "11": {
      "address": 11,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 100
      }
    },
    "12": {
      "address": 12,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "penManagers"
      }
    },
    "14": {
      "address": 14,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "15": {
      "address": 15,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_value__param"
      }
    },
    "16": {
      "address": 16,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__SetUsingDoubleClick"
      }
    },
    "17": {
      "address": 17,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "18": {
      "address": 18,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "19": {
      "address": 19,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__1_value__param"
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__SetEnabledLateSync"
      }
    },
    "21": {
      "address": 21,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__2_value__param"
      }
    },
    "23": {
      "address": 23,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__SetUsingSurftraceMode"
      }
    },
    "24": {
      "address": 24,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "29": {
      "address": 29,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "30": {
      "address": 30,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "31": {
      "address": 31,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "32": {
      "address": 32,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "33": {
      "address": 33,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "34": {
      "address": 34,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "35": {
      "address": 35,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "36": {
      "address": 36,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_LogicalXor__SystemBoolean_SystemBoolean__SystemBoolean"
      }
    },
    "39": {
      "address": 39,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseUtilities.__IsValid__SystemObject__SystemBoolean"
      }
    },
    "40": {
      "address": 40,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__SetActive__SystemBoolean__SystemVoid"
      }
    },
    "41": {
      "address": 41,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_UnaryNegation__SystemBoolean__SystemBoolean"
      }
    },
    "42": {
      "address": 42,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "43": {
      "address": 43,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariable__SystemString__SystemObject"
      }
    },
    "44": {
      "address": 44,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__get_Length__SystemInt32"
      }
    },
    "45": {
      "address": 45,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "46": {
      "address": 46,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Get__SystemInt32__SystemObject"
      }
    },
    "47": {
      "address": 47,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SetProgramVariable__SystemString_SystemObject__SystemVoid"
      }
    },
    "48": {
      "address": 48,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEvent__SystemString__SystemVoid"
      }
    },
    "49": {
      "address": 49,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    }
  }
}
```