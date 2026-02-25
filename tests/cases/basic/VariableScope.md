```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class VariableScope : UdonSharpBehaviour
{
    public int fieldVal = 10;

    public void Start()
    {
        int rootVar = 100;

        if (fieldVal > 5)
        {
            int branchVar = fieldVal * 2;
            rootVar += branchVar;
        }
        else
        {
            int branchVar = 0;
            rootVar -= branchVar;
        }

        int i = 0;
        while (i < 5)
        {
            int loopInternal = i * 10;
            rootVar += loopInternal;

            if (loopInternal > 20)
            {
                int deepVar = 1;
                rootVar += deepVar;
            }

            i = i + 1;
        }

        fieldVal = rootVar;
    }
}
```

```json
{
  "byteCodeHex": "00000001000000040000000100000005000000010000000C0000000900000001000000030000000100000006000000010000000D0000000600000014000000010000000D00000004000000940000000100000003000000010000000700000001000000100000000600000015000000010000000C0000000100000010000000010000000C000000060000001600000005000000C80000000100000008000000010000001000000009000000010000000C0000000100000010000000010000000C00000006000000170000000100000008000000010000000E00000009000000010000000E0000000100000006000000010000000F0000000600000018000000010000000F00000004000001D8000000010000000E000000010000000900000001000000110000000600000015000000010000000C0000000100000011000000010000000C00000006000000160000000100000011000000010000000A00000001000000120000000600000014000000010000001200000004000001B0000000010000000B000000010000001300000009000000010000000C0000000100000013000000010000000C0000000600000016000000010000000E000000010000000B000000010000000E000000060000001600000005000000DC000000010000000C0000000100000003000000090000000100000002000000090000000800000002",
  "byteCodeLength": 512,
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
    "fieldVal": {
      "name": "fieldVal",
      "type": "System.Int32",
      "address": 3
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 4
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 5
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 6
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 7
    },
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 8
    },
    "__const_SystemInt32_4": {
      "name": "__const_SystemInt32_4",
      "type": "System.Int32",
      "address": 9
    },
    "__const_SystemInt32_5": {
      "name": "__const_SystemInt32_5",
      "type": "System.Int32",
      "address": 10
    },
    "__const_SystemInt32_6": {
      "name": "__const_SystemInt32_6",
      "type": "System.Int32",
      "address": 11
    },
    "__lcl_rootVar_SystemInt32_0": {
      "name": "__lcl_rootVar_SystemInt32_0",
      "type": "System.Int32",
      "address": 12
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 13
    },
    "__lcl_i_SystemInt32_0": {
      "name": "__lcl_i_SystemInt32_0",
      "type": "System.Int32",
      "address": 14
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 15
    },
    "__lcl_branchVar_SystemInt32_0": {
      "name": "__lcl_branchVar_SystemInt32_0",
      "type": "System.Int32",
      "address": 16
    },
    "__lcl_loopInternal_SystemInt32_0": {
      "name": "__lcl_loopInternal_SystemInt32_0",
      "type": "System.Int32",
      "address": 17
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 18
    },
    "__lcl_deepVar_SystemInt32_0": {
      "name": "__lcl_deepVar_SystemInt32_0",
      "type": "System.Int32",
      "address": 19
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
        "value": 5084035386673785649
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VariableScope"
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
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "5": {
      "address": 5,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 100
      }
    },
    "6": {
      "address": 6,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 5
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
        "value": 0
      }
    },
    "9": {
      "address": 9,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 10
      }
    },
    "10": {
      "address": 10,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 20
      }
    },
    "11": {
      "address": 11,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "21": {
      "address": 21,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Multiplication__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "23": {
      "address": 23,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Subtraction__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "24": {
      "address": 24,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    }
  }
}
```
