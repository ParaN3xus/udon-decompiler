```csharp
using UdonSharp;
using UnityEngine;

public class ThisReference : UdonSharpBehaviour
{
    void Start()
    {
        UdonSharpBehaviour currentBehaviour = this;
        Debug.Log("Behaviour: " + currentBehaviour.name);

        GameObject currentGo = this.gameObject;
        Debug.Log("GameObject: " + currentGo.name);

        Transform currentTrans = this.transform;
        Debug.Log("Transform Position: " + currentTrans.position);
    }
}
```

```json
{
  "byteCodeHex": "00000001000000030000000100000004000000010000000A00000009000000010000000A000000010000000B00000006000000130000000100000005000000010000000B000000010000000C0000000600000014000000010000000C00000006000000150000000100000006000000010000000D00000009000000010000000D000000010000000E00000006000000130000000100000007000000010000000E000000010000000F0000000600000014000000010000000F000000060000001500000001000000080000000100000010000000090000000100000010000000010000001100000006000000160000000100000009000000010000001100000001000000120000000600000017000000010000001200000006000000150000000100000002000000090000000800000002",
  "byteCodeLength": 304,
  "symbols": {
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 3
    },
    "__this_VRCUdonUdonBehaviour_0": {
      "name": "__this_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 4
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 5
    },
    "__this_UnityEngineGameObject_0": {
      "name": "__this_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 6
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 7
    },
    "__this_UnityEngineTransform_0": {
      "name": "__this_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 8
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 9
    },
    "__lcl_currentBehaviour_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_currentBehaviour_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 10
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 11
    },
    "__intnl_SystemString_1": {
      "name": "__intnl_SystemString_1",
      "type": "System.String",
      "address": 12
    },
    "__lcl_currentGo_UnityEngineGameObject_0": {
      "name": "__lcl_currentGo_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 13
    },
    "__intnl_SystemString_2": {
      "name": "__intnl_SystemString_2",
      "type": "System.String",
      "address": 14
    },
    "__intnl_SystemString_3": {
      "name": "__intnl_SystemString_3",
      "type": "System.String",
      "address": 15
    },
    "__lcl_currentTrans_UnityEngineTransform_0": {
      "name": "__lcl_currentTrans_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 16
    },
    "__intnl_UnityEngineVector3_0": {
      "name": "__intnl_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 17
    },
    "__intnl_SystemString_4": {
      "name": "__intnl_SystemString_4",
      "type": "System.String",
      "address": 18
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
        "value": -8383024458406922570
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ThisReference"
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
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "4": {
      "address": 4,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "5": {
      "address": 5,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Behaviour: "
      }
    },
    "6": {
      "address": 6,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.GameObject, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "7": {
      "address": 7,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "GameObject: "
      }
    },
    "8": {
      "address": 8,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.Transform, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "9": {
      "address": 9,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Transform Position: "
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "12": {
      "address": 12,
      "type": "System.String",
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "15": {
      "address": 15,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "16": {
      "address": 16,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "17": {
      "address": 17,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "19": {
      "address": 19,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__get_name__SystemString"
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__op_Addition__SystemString_SystemString__SystemString"
      }
    },
    "21": {
      "address": 21,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_position__UnityEngineVector3"
      }
    },
    "23": {
      "address": 23,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Concat__SystemObject_SystemObject__SystemString"
      }
    }
  }
}
```
