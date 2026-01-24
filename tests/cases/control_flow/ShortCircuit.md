```csharp
using UdonSharp;
using UnityEngine;

public class ShortCircuit : UdonSharpBehaviour
{
    public int a = 10;
    public int b = 20;
    public int c = 30;

    public override void Interact()
    {
        if (a > 0 && b < 100)
        {
            Debug.Log("AND Check Passed");
        }
        if (a == 5 || b == 5)
        {
            Debug.Log("OR Check Passed");
        }
        if (a > b && (b > c || a == 10))
        {
            Debug.Log("Mixed Check Passed");
        }
    }
}
```

```json
{
  "byteCodeHex": "000000010000000600000001000000030000000100000007000000010000000E0000000600000012000000010000000E000000040000005800000001000000040000000100000008000000010000000E0000000600000013000000010000000E0000000400000078000000010000000900000006000000140000000100000003000000010000000A000000010000000F0000000600000015000000010000000F00000004000000B000000005000000D00000000100000004000000010000000A000000010000000F0000000600000015000000010000000F00000004000000F0000000010000000B000000060000001400000001000000030000000100000004000000010000001000000006000000120000000100000010000000040000018C00000001000000040000000100000005000000010000001100000006000000120000000100000011000000040000015800000005000001780000000100000003000000010000000C000000010000001100000006000000150000000100000011000000010000001000000009000000010000001000000004000001AC000000010000000D00000006000000140000000100000002000000090000000800000002",
  "byteCodeLength": 448,
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
    "a": {
      "name": "a",
      "type": "System.Int32",
      "address": 3
    },
    "b": {
      "name": "b",
      "type": "System.Int32",
      "address": 4
    },
    "c": {
      "name": "c",
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
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 9
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 10
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 11
    },
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 12
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 13
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 14
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 15
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 16
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 17
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
        "value": -1492630976521771770
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ShortCircuit"
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
        "value": 20
      }
    },
    "5": {
      "address": 5,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 30
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
        "value": 0
      }
    },
    "8": {
      "address": 8,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 100
      }
    },
    "9": {
      "address": 9,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "AND Check Passed"
      }
    },
    "10": {
      "address": 10,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 5
      }
    },
    "11": {
      "address": 11,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "OR Check Passed"
      }
    },
    "12": {
      "address": 12,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 10
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Mixed Check Passed"
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "17": {
      "address": 17,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "19": {
      "address": 19,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    },
    "21": {
      "address": 21,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    }
  }
}
```
