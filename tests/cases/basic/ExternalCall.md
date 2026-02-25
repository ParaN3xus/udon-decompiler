```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class ExternalCall : UdonSharpBehaviour
{
    void Start()
    {
        UnityEngine.Random.InitState(12345);
        int absValue = Mathf.Abs(-99);
        this.gameObject.SetActive(true);
        Transform child = this.transform.GetChild(0);
    }
}
```

```json
{
  "byteCodeHex": "00000001000000030000000100000004000000060000000C0000000100000005000000010000000A000000060000000D00000001000000060000000100000007000000060000000E00000001000000080000000100000009000000010000000B000000060000000F0000000100000002000000090000000800000002",
  "byteCodeLength": 124,
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
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 4
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 5
    },
    "__this_UnityEngineGameObject_0": {
      "name": "__this_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 6
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 7
    },
    "__this_UnityEngineTransform_0": {
      "name": "__this_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 8
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 9
    },
    "__lcl_absValue_SystemInt32_0": {
      "name": "__lcl_absValue_SystemInt32_0",
      "type": "System.Int32",
      "address": 10
    },
    "__lcl_child_UnityEngineTransform_0": {
      "name": "__lcl_child_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 11
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
        "value": 634558263974105786
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ExternalCall"
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 12345
      }
    },
    "5": {
      "address": 5,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": -99
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "10": {
      "address": 10,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "11": {
      "address": 11,
      "type": "UnityEngine.Transform",
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
        "value": "UnityEngineRandom.__InitState__SystemInt32__SystemVoid"
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__Abs__SystemInt32__SystemInt32"
      }
    },
    "14": {
      "address": 14,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__SetActive__SystemBoolean__SystemVoid"
      }
    },
    "15": {
      "address": 15,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__GetChild__SystemInt32__UnityEngineTransform"
      }
    }
  }
}
```
