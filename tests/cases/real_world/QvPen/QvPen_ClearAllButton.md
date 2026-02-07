<!-- ci: skip-compile -->

```csharp
using UdonSharp;
using UnityEngine;

namespace QvPen.UdonScript.UI
{
    [AddComponentMenu("")]
    [DefaultExecutionOrder(30)]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_ClearAllButton : UdonSharpBehaviour
    {
        [SerializeField]
        private QvPen_Settings settings;

        public override void Interact()
        {
            foreach (var penManager in settings.penManagers)
                if (penManager)
                    penManager.Clear();
        }
    }
}
```

```json
{
  "byteCodeHex": "000000010000000400000001000000030000000100000005000000010000000900000006000000100000000100000009000000010000000A00000009000000010000000A000000010000000B00000006000000110000000100000006000000010000000C00000009000000010000000C000000010000000B000000010000000D0000000600000012000000010000000D0000000400000120000000010000000A000000010000000C000000010000000E0000000600000013000000010000000E000000010000000F0000000600000014000000010000000F00000004000000F8000000010000000E00000001000000070000000600000015000000010000000C0000000100000008000000010000000C000000060000001600000005000000680000000100000002000000090000000800000002",
  "byteCodeLength": 308,
  "symbols": {
    "__intnl_SystemObject_0": {
      "name": "__intnl_SystemObject_0",
      "type": "System.Object",
      "address": 9
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 5
    },
    "__lcl_penManager_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_penManager_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 14
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 4
    },
    "__intnl_UnityEngineComponentArray_0": {
      "name": "__intnl_UnityEngineComponentArray_0",
      "type": "UnityEngine.Component[]",
      "address": 10
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
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
      "address": 12
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 11
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 7
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 8
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 6
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 13
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 15
    }
  },
  "entryPoints": [
    {
      "name": "_interact",
      "address": 0
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": -6896214873689741659
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.UI.QvPen_ClearAllButton"
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
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "5": {
      "address": 5,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "penManagers"
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Clear"
      }
    },
    "8": {
      "address": 8,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "9": {
      "address": 9,
      "type": "System.Object",
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
        "value": null
      }
    },
    "11": {
      "address": 11,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "12": {
      "address": 12,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "13": {
      "address": 13,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "14": {
      "address": 14,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "15": {
      "address": 15,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "16": {
      "address": 16,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariable__SystemString__SystemObject"
      }
    },
    "17": {
      "address": 17,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__get_Length__SystemInt32"
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "19": {
      "address": 19,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Get__SystemInt32__SystemObject"
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__op_Implicit__UnityEngineObject__SystemBoolean"
      }
    },
    "21": {
      "address": 21,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEvent__SystemString__SystemVoid"
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    }
  }
}
```
