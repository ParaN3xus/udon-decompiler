```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class For : UdonSharpBehaviour
{
    public void Start()
    {
        int[] numbers = new int[] { 1, 2, 3, 4, 5 };
        int total = 0;

        for (int i = 0; i < numbers.Length; i++)
        {
            total += numbers[i];
            Debug.Log(i);
        }

        Debug.Log(total);
    }
}
```

```json
{
  "byteCodeHex": "00000001000000030000000100000004000000010000000A0000000600000010000000010000000A000000010000000500000001000000060000000600000011000000010000000A000000010000000600000001000000070000000600000011000000010000000A000000010000000700000001000000080000000600000011000000010000000A000000010000000800000001000000090000000600000011000000010000000A0000000100000009000000010000000400000006000000110000000100000005000000010000000B000000090000000100000005000000010000000C00000009000000010000000A000000010000000D0000000600000012000000010000000C000000010000000D000000010000000E0000000600000013000000010000000E00000004000001A8000000010000000A000000010000000C000000010000000F0000000600000014000000010000000B000000010000000F000000010000000B0000000600000015000000010000000C0000000600000016000000010000000C0000000100000006000000010000000C000000060000001500000005000000E8000000010000000B00000006000000160000000100000002000000090000000800000002",
  "byteCodeLength": 460,
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
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 6
    },
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 7
    },
    "__const_SystemInt32_4": {
      "name": "__const_SystemInt32_4",
      "type": "System.Int32",
      "address": 8
    },
    "__const_SystemInt32_5": {
      "name": "__const_SystemInt32_5",
      "type": "System.Int32",
      "address": 9
    },
    "__lcl_numbers_SystemInt32Array_0": {
      "name": "__lcl_numbers_SystemInt32Array_0",
      "type": "System.Int32[]",
      "address": 10
    },
    "__lcl_total_SystemInt32_0": {
      "name": "__lcl_total_SystemInt32_0",
      "type": "System.Int32",
      "address": 11
    },
    "__lcl_i_SystemInt32_0": {
      "name": "__lcl_i_SystemInt32_0",
      "type": "System.Int32",
      "address": 12
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 13
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 14
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 15
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
        "value": 1395056345059366346
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "For"
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
        "value": 5
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "9": {
      "address": 9,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 4
      }
    },
    "10": {
      "address": 10,
      "type": "System.Int32[]",
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "16": {
      "address": 16,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32Array.__ctor__SystemInt32__SystemInt32Array"
      }
    },
    "17": {
      "address": 17,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32Array.__Set__SystemInt32_SystemInt32__SystemVoid"
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__get_Length__SystemInt32"
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
        "value": "SystemInt32Array.__Get__SystemInt32__SystemInt32"
      }
    },
    "21": {
      "address": 21,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    }
  }
}
```
