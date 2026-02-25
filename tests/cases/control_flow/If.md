```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class If : UdonSharpBehaviour
{
    public void Start()
    {
        int a = 10;
        if (a > 5)
        {
            Debug.Log("Condition is true");
            a = a * 2;
        }

        Debug.Log("Finished");
    }
}
```

```json
{
  "byteCodeHex": "0000000100000003000000010000000400000001000000090000000900000001000000090000000100000005000000010000000A000000060000000B000000010000000A000000040000007C0000000100000006000000060000000C000000010000000900000001000000070000000100000009000000060000000D0000000100000008000000060000000C0000000100000002000000090000000800000002",
  "byteCodeLength": 160,
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
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 6
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 7
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 8
    },
    "__lcl_a_SystemInt32_0": {
      "name": "__lcl_a_SystemInt32_0",
      "type": "System.Int32",
      "address": 9
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 10
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
        "value": -5963752566360688098
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "If"
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
        "value": 10
      }
    },
    "5": {
      "address": 5,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 5
      }
    },
    "6": {
      "address": 6,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Condition is true"
      }
    },
    "7": {
      "address": 7,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "8": {
      "address": 8,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Finished"
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "11": {
      "address": 11,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "12": {
      "address": 12,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Multiplication__SystemInt32_SystemInt32__SystemInt32"
      }
    }
  }
}
```
