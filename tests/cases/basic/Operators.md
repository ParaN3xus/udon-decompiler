```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class Operators : UdonSharpBehaviour
{
    void Start()
    {
        // arithmetic
        int a = 10;
        int b = 3;
        int add = a + b;
        int sub = a - b;
        int mul = a * b;
        int div = a / b;
        int mod = a % b;
        int neg = -a;

        // logical
        bool flag = true;
        bool notFlag = !flag;

        // bits
        int bitNot = ~a;
        int bitAnd = a & b;
        int bitOr = a | b;
        int bitXor = a ^ b;
        int lShift = a << 1;
        int rShift = a >> 1;

        // compare
        bool eq = (a == b);
        bool neq = (a != b);
        bool gt = (a > b);
        bool gte = (a >= b);
        bool lt = (a < b);
        bool lte = (a <= b);
    }
}
```

```json
{
  "byteCodeHex": "000000010000000300000001000000040000000100000009000000090000000100000005000000010000000A000000090000000100000009000000010000000A000000010000000B000000060000001F0000000100000009000000010000000A000000010000000C00000006000000200000000100000009000000010000000A000000010000000D00000006000000210000000100000009000000010000000A000000010000000E00000006000000220000000100000009000000010000000A000000010000000F0000000600000023000000010000000900000001000000100000000600000024000000010000000600000001000000110000000900000001000000110000000100000012000000060000002500000001000000090000000100000007000000010000001300000006000000260000000100000009000000010000000A000000010000001400000006000000270000000100000009000000010000000A000000010000001500000006000000280000000100000009000000010000000A000000010000001600000006000000260000000100000009000000010000000800000001000000170000000600000029000000010000000900000001000000080000000100000018000000060000002A0000000100000009000000010000000A0000000100000019000000060000002B0000000100000009000000010000000A000000010000001A000000060000002C0000000100000009000000010000000A000000010000001B000000060000002D0000000100000009000000010000000A000000010000001C000000060000002E0000000100000009000000010000000A000000010000001D000000060000002F0000000100000009000000010000000A000000010000001E00000006000000300000000100000002000000090000000800000002",
  "byteCodeLength": 680,
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
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
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
    "__lcl_a_SystemInt32_0": {
      "name": "__lcl_a_SystemInt32_0",
      "type": "System.Int32",
      "address": 9
    },
    "__lcl_b_SystemInt32_0": {
      "name": "__lcl_b_SystemInt32_0",
      "type": "System.Int32",
      "address": 10
    },
    "__lcl_add_SystemInt32_0": {
      "name": "__lcl_add_SystemInt32_0",
      "type": "System.Int32",
      "address": 11
    },
    "__lcl_sub_SystemInt32_0": {
      "name": "__lcl_sub_SystemInt32_0",
      "type": "System.Int32",
      "address": 12
    },
    "__lcl_mul_SystemInt32_0": {
      "name": "__lcl_mul_SystemInt32_0",
      "type": "System.Int32",
      "address": 13
    },
    "__lcl_div_SystemInt32_0": {
      "name": "__lcl_div_SystemInt32_0",
      "type": "System.Int32",
      "address": 14
    },
    "__lcl_mod_SystemInt32_0": {
      "name": "__lcl_mod_SystemInt32_0",
      "type": "System.Int32",
      "address": 15
    },
    "__lcl_neg_SystemInt32_0": {
      "name": "__lcl_neg_SystemInt32_0",
      "type": "System.Int32",
      "address": 16
    },
    "__lcl_flag_SystemBoolean_0": {
      "name": "__lcl_flag_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 17
    },
    "__lcl_notFlag_SystemBoolean_0": {
      "name": "__lcl_notFlag_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 18
    },
    "__lcl_bitNot_SystemInt32_0": {
      "name": "__lcl_bitNot_SystemInt32_0",
      "type": "System.Int32",
      "address": 19
    },
    "__lcl_bitAnd_SystemInt32_0": {
      "name": "__lcl_bitAnd_SystemInt32_0",
      "type": "System.Int32",
      "address": 20
    },
    "__lcl_bitOr_SystemInt32_0": {
      "name": "__lcl_bitOr_SystemInt32_0",
      "type": "System.Int32",
      "address": 21
    },
    "__lcl_bitXor_SystemInt32_0": {
      "name": "__lcl_bitXor_SystemInt32_0",
      "type": "System.Int32",
      "address": 22
    },
    "__lcl_lShift_SystemInt32_0": {
      "name": "__lcl_lShift_SystemInt32_0",
      "type": "System.Int32",
      "address": 23
    },
    "__lcl_rShift_SystemInt32_0": {
      "name": "__lcl_rShift_SystemInt32_0",
      "type": "System.Int32",
      "address": 24
    },
    "__lcl_eq_SystemBoolean_0": {
      "name": "__lcl_eq_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 25
    },
    "__lcl_neq_SystemBoolean_0": {
      "name": "__lcl_neq_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 26
    },
    "__lcl_gt_SystemBoolean_0": {
      "name": "__lcl_gt_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 27
    },
    "__lcl_gte_SystemBoolean_0": {
      "name": "__lcl_gte_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 28
    },
    "__lcl_lt_SystemBoolean_0": {
      "name": "__lcl_lt_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 29
    },
    "__lcl_lte_SystemBoolean_0": {
      "name": "__lcl_lte_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 30
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
        "value": -8776592101424770004
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Operators"
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
        "value": 3
      }
    },
    "6": {
      "address": 6,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "7": {
      "address": 7,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": -1
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "30": {
      "address": 30,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "31": {
      "address": 31,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "32": {
      "address": 32,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Subtraction__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "33": {
      "address": 33,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Multiplication__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "34": {
      "address": 34,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Division__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "35": {
      "address": 35,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Remainder__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "36": {
      "address": 36,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_UnaryMinus__SystemInt32__SystemInt32"
      }
    },
    "37": {
      "address": 37,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_UnaryNegation__SystemBoolean__SystemBoolean"
      }
    },
    "38": {
      "address": 38,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LogicalXor__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "39": {
      "address": 39,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LogicalAnd__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "40": {
      "address": 40,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LogicalOr__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "41": {
      "address": 41,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LeftShift__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "42": {
      "address": 42,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_RightShift__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "43": {
      "address": 43,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "44": {
      "address": 44,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Inequality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "45": {
      "address": 45,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "46": {
      "address": 46,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "47": {
      "address": 47,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "48": {
      "address": 48,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    }
  }
}
```
