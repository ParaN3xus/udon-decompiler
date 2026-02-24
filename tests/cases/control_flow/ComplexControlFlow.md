```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class ComplexControlFlow : UdonSharpBehaviour
{
    public int limit = 5;
    public int seed = 3;

    public void Start()
    {
        int i = 0;
        int j = 0;
        int sum = 0;
        bool flag = false;

        do
        {
            int local = i * seed;

            if (local % 2 == 0 && i < limit)
            {
                sum += local;
            }
            else if (local % 3 == 0 || i == 0)
            {
                sum -= local;
            }
            else
            {
                sum += 1;
            }

            j = 0;
            while (j < 4)
            {
                if (j == 2)
                {
                    j++;
                    continue;
                }

                sum += j;

                if (sum > 50)
                {
                    flag = true;
                    break;
                }

                j++;
            }

            if (flag)
            {
                break;
            }

            switch (i)
            {
                case 0:
                    sum += 10;
                    break;
                case 1:
                case 2:
                    sum += 20;
                    break;
                default:
                    sum += 30;
                    break;
            }

            for (int k = 0; k < 3; k++)
            {
                if (k == 1)
                {
                    continue;
                }
                sum += k;
            }

            i++;
        } while (i < limit);

        if (sum > 0 ? true : false)
        {
            Debug.Log("Positive sum");
        }
        else
        {
            Debug.Log("Non-positive sum");
        }

        if (sum > 100)
        {
            Debug.Log("Large sum");
            return;
        }

        Debug.Log(sum);
    }
}
```

```json
{
  "byteCodeHex": "0000000100000005000000010000000600000001000000150000000900000001000000060000000100000016000000090000000100000006000000010000001700000009000000010000000700000001000000180000000900000001000000150000000100000004000000010000001D0000000600000023000000010000001D0000000100000008000000010000001E0000000600000024000000010000001E000000010000000600000001000000190000000600000025000000010000001900000004000000E80000000100000015000000010000000300000001000000190000000600000026000000010000001900000004000001200000000100000017000000010000001D0000000100000017000000060000002700000005000001F0000000010000001D0000000100000009000000010000001F0000000600000024000000010000001F0000000100000006000000010000001A0000000600000025000000010000001A0000000400000178000000050000019800000001000000150000000100000006000000010000001A0000000600000025000000010000001A00000004000001D00000000100000017000000010000001D0000000100000017000000060000002800000005000001F00000000100000017000000010000000A0000000100000017000000060000002700000001000000060000000100000016000000090000000100000016000000010000000B000000010000001B0000000600000026000000010000001B000000040000032000000001000000160000000100000008000000010000001C0000000600000025000000010000001C000000040000028C0000000100000016000000010000000A00000001000000160000000600000027000000050000020400000001000000170000000100000016000000010000001700000006000000270000000100000017000000010000000C00000001000000200000000600000029000000010000002000000004000002F8000000010000000D00000001000000180000000900000005000003200000000100000016000000010000000A00000001000000160000000600000027000000050000020400000001000000180000000400000338000000050000057C00000001000000150000000100000006000000010000001C0000000600000025000000010000001C00000004000003900000000100000017000000010000000E0000000100000017000000060000002700000005000004600000000100000015000000010000000A00000001000000200000000600000025000000010000002000000004000003C8000000050000040000000001000000150000000100000008000000010000002100000006000000250000000100000021000000040000045800000005000004000000000100000017000000010000000F0000000100000017000000060000002700000005000004600000000500000458000000010000001700000001000000100000000100000017000000060000002700000005000004600000000500000430000000010000000600000001000000220000000900000001000000220000000100000009000000010000001C0000000600000026000000010000001C00000004000005240000000100000022000000010000000A00000001000000200000000600000025000000010000002000000004000004DC00000005000004FC00000001000000170000000100000022000000010000001700000006000000270000000100000022000000010000000A0000000100000022000000060000002700000005000004740000000100000015000000010000000A0000000100000015000000060000002700000001000000150000000100000003000000010000001900000006000000260000000100000019000000040000057C000000050000005800000001000000170000000100000006000000010000001A0000000600000029000000010000001A00000004000005C8000000010000000D000000010000001B0000000900000005000005DC0000000100000007000000010000001B00000009000000010000001B00000004000006040000000100000011000000060000002A00000005000006140000000100000012000000060000002A00000001000000170000000100000013000000010000001C0000000600000029000000010000001C00000004000006680000000100000014000000060000002A00000001000000020000000900000008000000020000000100000017000000060000002A0000000100000002000000090000000800000002",
  "byteCodeLength": 1676,
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
    "limit": {
      "name": "limit",
      "type": "System.Int32",
      "address": 3
    },
    "seed": {
      "name": "seed",
      "type": "System.Int32",
      "address": 4
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 5
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 6
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
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
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 10
    },
    "__const_SystemInt32_4": {
      "name": "__const_SystemInt32_4",
      "type": "System.Int32",
      "address": 11
    },
    "__const_SystemInt32_5": {
      "name": "__const_SystemInt32_5",
      "type": "System.Int32",
      "address": 12
    },
    "__const_SystemBoolean_1": {
      "name": "__const_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 13
    },
    "__const_SystemInt32_6": {
      "name": "__const_SystemInt32_6",
      "type": "System.Int32",
      "address": 14
    },
    "__const_SystemInt32_7": {
      "name": "__const_SystemInt32_7",
      "type": "System.Int32",
      "address": 15
    },
    "__const_SystemInt32_8": {
      "name": "__const_SystemInt32_8",
      "type": "System.Int32",
      "address": 16
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 17
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 18
    },
    "__const_SystemInt32_9": {
      "name": "__const_SystemInt32_9",
      "type": "System.Int32",
      "address": 19
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 20
    },
    "__lcl_i_SystemInt32_0": {
      "name": "__lcl_i_SystemInt32_0",
      "type": "System.Int32",
      "address": 21
    },
    "__lcl_j_SystemInt32_0": {
      "name": "__lcl_j_SystemInt32_0",
      "type": "System.Int32",
      "address": 22
    },
    "__lcl_sum_SystemInt32_0": {
      "name": "__lcl_sum_SystemInt32_0",
      "type": "System.Int32",
      "address": 23
    },
    "__lcl_flag_SystemBoolean_0": {
      "name": "__lcl_flag_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 24
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 25
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 26
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 27
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 28
    },
    "__lcl_local_SystemInt32_0": {
      "name": "__lcl_local_SystemInt32_0",
      "type": "System.Int32",
      "address": 29
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 30
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 31
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 32
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 33
    },
    "__lcl_k_SystemInt32_0": {
      "name": "__lcl_k_SystemInt32_0",
      "type": "System.Int32",
      "address": 34
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
        "value": -6589313354780580882
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ComplexControlFlow"
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
        "value": 5
      }
    },
    "4": {
      "address": 4,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "5": {
      "address": 5,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "8": {
      "address": 8,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "9": {
      "address": 9,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "10": {
      "address": 10,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "11": {
      "address": 11,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 4
      }
    },
    "12": {
      "address": 12,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 50
      }
    },
    "13": {
      "address": 13,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "14": {
      "address": 14,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 10
      }
    },
    "15": {
      "address": 15,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 20
      }
    },
    "16": {
      "address": 16,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 30
      }
    },
    "17": {
      "address": 17,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Positive sum"
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Non-positive sum"
      }
    },
    "19": {
      "address": 19,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 100
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Large sum"
      }
    },
    "21": {
      "address": 21,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "22": {
      "address": 22,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "23": {
      "address": 23,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "24": {
      "address": 24,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "25": {
      "address": 25,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "26": {
      "address": 26,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "27": {
      "address": 27,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "28": {
      "address": 28,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "29": {
      "address": 29,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "30": {
      "address": 30,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "31": {
      "address": 31,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "32": {
      "address": 32,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "33": {
      "address": 33,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "34": {
      "address": 34,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "35": {
      "address": 35,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Multiplication__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "36": {
      "address": 36,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Remainder__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "37": {
      "address": 37,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "38": {
      "address": 38,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "39": {
      "address": 39,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "40": {
      "address": 40,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Subtraction__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "41": {
      "address": 41,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "42": {
      "address": 42,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    }
  }
}
```
