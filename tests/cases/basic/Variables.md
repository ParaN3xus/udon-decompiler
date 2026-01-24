```csharp
using UdonSharp;
using UnityEngine;

public class Variables : UdonSharpBehaviour
{
    public int count = 3;
    public float speed = 2.5f;
    public bool enabledFlag = true;

    private int _ticks;

    private void Update()
    {
        if (!enabledFlag)
        {
            return;
        }

        _ticks += 1;
        transform.Rotate(0f, speed, 0f);

        if (_ticks >= count)
        {
            enabledFlag = false;
        }
    }
}
```

```json
{
  "byteCodeHex": "00000001000000070000000100000005000000040000002000000005000000340000000100000002000000090000000800000002000000010000000600000001000000080000000100000006000000060000000D0000000100000009000000010000000A0000000100000004000000010000000A000000060000000E00000001000000060000000100000003000000010000000C000000060000000F000000010000000C00000004000000C0000000010000000B0000000100000005000000090000000100000002000000090000000800000002",
  "byteCodeLength": 212,
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
    "count": {
      "name": "count",
      "type": "System.Int32",
      "address": 3
    },
    "speed": {
      "name": "speed",
      "type": "System.Single",
      "address": 4
    },
    "enabledFlag": {
      "name": "enabledFlag",
      "type": "System.Boolean",
      "address": 5
    },
    "_ticks": {
      "name": "_ticks",
      "type": "System.Int32",
      "address": 6
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 7
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 8
    },
    "__this_UnityEngineTransform_0": {
      "name": "__this_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 9
    },
    "__const_SystemSingle_0": {
      "name": "__const_SystemSingle_0",
      "type": "System.Single",
      "address": 10
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 11
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 12
    }
  },
  "entryPoints": [
    {
      "name": "_update",
      "address": 0
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": -3533104944274613502
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Variables"
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "4": {
      "address": 4,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 2.5
      }
    },
    "5": {
      "address": 5,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
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
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
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
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.Transform, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "10": {
      "address": 10,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "11": {
      "address": 11,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "14": {
      "address": 14,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__Rotate__SystemSingle_SystemSingle_SystemSingle__SystemVoid"
      }
    },
    "15": {
      "address": 15,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    }
  }
}
```
