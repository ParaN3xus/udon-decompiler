<!-- ci: skip-compile -->

```csharp
using UdonSharp;
using UnityEngine;

namespace QvPen.UdonScript.UI
{
    [AddComponentMenu("")]
    [DefaultExecutionOrder(30)]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_ShowOrHideButton : UdonSharpBehaviour
    {
        [SerializeField]
        private GameObject[] gameObjects = { };

        [SerializeField]
        private bool isShown = true;

        [SerializeField]
        private GameObject displayObjectOn;
        [SerializeField]
        private GameObject displayObjectOff;

        private void Start() => UpdateActivity();

        public override void Interact()
        {
            isShown ^= true;
            UpdateActivity();
        }

        private void UpdateActivity()
        {
            if (displayObjectOn)
                displayObjectOn.SetActive(isShown);
            if (displayObjectOff)
                displayObjectOff.SetActive(!isShown);

            foreach (var go in gameObjects)
                if (go)
                    go.SetActive(isShown);
        }
    }
}
```

```json
{
  "byteCodeHex": "000000010000000700000001000000080000000500000080000000010000000200000009000000080000000200000001000000070000000100000004000000010000000900000001000000040000000600000015000000010000000A0000000500000080000000010000000200000009000000080000000200000001000000070000000100000005000000010000000D0000000600000016000000010000000D00000004000000C00000000100000005000000010000000400000006000000170000000100000006000000010000000E0000000600000016000000010000000E00000004000001180000000100000004000000010000000F00000006000000180000000100000006000000010000000F0000000600000017000000010000000300000001000000100000000600000019000000010000000B000000010000001100000009000000010000001100000001000000100000000100000012000000060000001A000000010000001200000004000001FC000000010000000300000001000000110000000100000013000000060000001B000000010000001300000001000000140000000600000016000000010000001400000004000001D40000000100000013000000010000000400000006000000170000000100000011000000010000000C0000000100000011000000060000001C00000005000001440000000100000002000000090000000800000002",
  "byteCodeLength": 528,
  "symbols": {
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 10
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 8
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 9
    },
    "displayObjectOff": {
      "name": "displayObjectOff",
      "type": "UnityEngine.GameObject",
      "address": 6
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 7
    },
    "gameObjects": {
      "name": "gameObjects",
      "type": "UnityEngine.GameObject[]",
      "address": 3
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "displayObjectOn": {
      "name": "displayObjectOn",
      "type": "UnityEngine.GameObject",
      "address": 5
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 17
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 16
    },
    "__lcl_go_UnityEngineGameObject_0": {
      "name": "__lcl_go_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 19
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 12
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 11
    },
    "isShown": {
      "name": "isShown",
      "type": "System.Boolean",
      "address": 4
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 13
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 14
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 15
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 18
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 20
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
        "value": 3446653693432281303
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.UI.QvPen_ShowOrHideButton"
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
      "type": "UnityEngine.GameObject[]",
      "value": {
        "isSerializable": true,
        "value": []
      }
    },
    "4": {
      "address": 4,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
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
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "8": {
      "address": 8,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24
      }
    },
    "9": {
      "address": 9,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "10": {
      "address": 10,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 100
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
        "value": 1
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "17": {
      "address": 17,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "18": {
      "address": 18,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "19": {
      "address": 19,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "20": {
      "address": 20,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "21": {
      "address": 21,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_LogicalXor__SystemBoolean_SystemBoolean__SystemBoolean"
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__op_Implicit__UnityEngineObject__SystemBoolean"
      }
    },
    "23": {
      "address": 23,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__SetActive__SystemBoolean__SystemVoid"
      }
    },
    "24": {
      "address": 24,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_UnaryNegation__SystemBoolean__SystemBoolean"
      }
    },
    "25": {
      "address": 25,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__get_Length__SystemInt32"
      }
    },
    "26": {
      "address": 26,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "27": {
      "address": 27,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Get__SystemInt32__SystemObject"
      }
    },
    "28": {
      "address": 28,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    }
  }
}
```