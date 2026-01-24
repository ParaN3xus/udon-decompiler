```csharp
using UdonSharp;
using UnityEngine;

public class Branching : UdonSharpBehaviour
{
    public int threshold = 10;
    public int value = 7;

    public int Evaluate()
    {
        if (value > threshold)
        {
            return 1;
        }
        else if (value == threshold)
        {
            return 0;
        }

        return -1;
    }
}
```

```json
{
  "byteCodeHex": "000000010000000600000001000000040000000100000003000000010000000A000000060000000C000000010000000A00000004000000680000000100000007000000010000000500000009000000010000000200000009000000080000000200000005000000C000000001000000040000000100000003000000010000000B000000060000000D000000010000000B00000004000000C000000001000000080000000100000005000000090000000100000002000000090000000800000002000000010000000900000001000000050000000900000001000000020000000900000008000000020000000100000002000000090000000800000002",
  "byteCodeLength": 252,
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
    "threshold": {
      "name": "threshold",
      "type": "System.Int32",
      "address": 3
    },
    "value": {
      "name": "value",
      "type": "System.Int32",
      "address": 4
    },
    "__0_Evaluate__ret": {
      "name": "__0_Evaluate__ret",
      "type": "System.Int32",
      "address": 5
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 6
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 7
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 8
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 9
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 10
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 11
    }
  },
  "entryPoints": [
    {
      "name": "Evaluate",
      "address": 0
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": -5157612155080300646
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Branching"
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
        "value": 10
      }
    },
    "4": {
      "address": 4,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 7
      }
    },
    "5": {
      "address": 5,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "6": {
      "address": 6,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "7": {
      "address": 7,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "8": {
      "address": 8,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "9": {
      "address": 9,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": -1
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "12": {
      "address": 12,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    }
  }
}
```
