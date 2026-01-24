```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class ExpressionInline : UdonSharpBehaviour
{
    public void Start()
    {
        int a = 10;
        int b = 20;
        float f = 2.5f;

        int res1 = (a + b) * (b - a);
        float res2 = (a * 0.5f) + (f / 2.0f);
        Vector3 v = Vector3.up * f + new Vector3(1, 0, 0);
        if ((a > 5 && b < 30) || res1 == 0)
        {
            string n = this.transform.parent.name;
            Debug.Log(Mathf.Abs(f).ToString());
        }
    }
}
```

```json
{
  "byteCodeHex": "00000001000000030000000100000004000000010000000F0000000900000001000000050000000100000010000000090000000100000006000000010000001100000009000000010000000F0000000100000010000000010000001300000006000000210000000100000010000000010000000F000000010000001400000006000000220000000100000013000000010000001400000001000000120000000600000023000000010000000F0000000100000016000000060000002400000001000000160000000100000007000000010000001700000006000000250000000100000011000000010000000800000001000000180000000600000026000000010000001700000001000000180000000100000015000000060000002700000001000000090000000100000011000000010000001A0000000600000028000000010000001A000000010000000A00000001000000190000000600000029000000010000000F000000010000000B000000010000001C000000060000002A000000010000001C00000004000001AC0000000100000010000000010000000C000000010000001C000000060000002B000000010000001C000000010000001B00000009000000010000001B00000004000001D800000005000001F80000000100000012000000010000000D000000010000001B000000060000002C000000010000001B0000000400000278000000010000000E000000010000001E000000060000002D000000010000001E000000010000001D000000060000002E0000000100000011000000010000001F000000060000002F000000010000001F00000001000000200000000600000030000000010000002000000006000000310000000100000002000000090000000800000002",
  "byteCodeLength": 652,
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
    "__const_SystemSingle_0": {
      "name": "__const_SystemSingle_0",
      "type": "System.Single",
      "address": 6
    },
    "__const_SystemSingle_1": {
      "name": "__const_SystemSingle_1",
      "type": "System.Single",
      "address": 7
    },
    "__const_SystemSingle_2": {
      "name": "__const_SystemSingle_2",
      "type": "System.Single",
      "address": 8
    },
    "__const_UnityEngineVector3_0": {
      "name": "__const_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 9
    },
    "__const_UnityEngineVector3_1": {
      "name": "__const_UnityEngineVector3_1",
      "type": "UnityEngine.Vector3",
      "address": 10
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 11
    },
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 12
    },
    "__const_SystemInt32_4": {
      "name": "__const_SystemInt32_4",
      "type": "System.Int32",
      "address": 13
    },
    "__this_UnityEngineTransform_0": {
      "name": "__this_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 14
    },
    "__lcl_a_SystemInt32_0": {
      "name": "__lcl_a_SystemInt32_0",
      "type": "System.Int32",
      "address": 15
    },
    "__lcl_b_SystemInt32_0": {
      "name": "__lcl_b_SystemInt32_0",
      "type": "System.Int32",
      "address": 16
    },
    "__lcl_f_SystemSingle_0": {
      "name": "__lcl_f_SystemSingle_0",
      "type": "System.Single",
      "address": 17
    },
    "__lcl_res1_SystemInt32_0": {
      "name": "__lcl_res1_SystemInt32_0",
      "type": "System.Int32",
      "address": 18
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 19
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 20
    },
    "__lcl_res2_SystemSingle_0": {
      "name": "__lcl_res2_SystemSingle_0",
      "type": "System.Single",
      "address": 21
    },
    "__intnl_SystemSingle_0": {
      "name": "__intnl_SystemSingle_0",
      "type": "System.Single",
      "address": 22
    },
    "__intnl_SystemSingle_1": {
      "name": "__intnl_SystemSingle_1",
      "type": "System.Single",
      "address": 23
    },
    "__intnl_SystemSingle_2": {
      "name": "__intnl_SystemSingle_2",
      "type": "System.Single",
      "address": 24
    },
    "__lcl_v_UnityEngineVector3_0": {
      "name": "__lcl_v_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 25
    },
    "__intnl_UnityEngineVector3_0": {
      "name": "__intnl_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 26
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 27
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 28
    },
    "__lcl_n_SystemString_0": {
      "name": "__lcl_n_SystemString_0",
      "type": "System.String",
      "address": 29
    },
    "__intnl_UnityEngineTransform_0": {
      "name": "__intnl_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 30
    },
    "__intnl_SystemSingle_3": {
      "name": "__intnl_SystemSingle_3",
      "type": "System.Single",
      "address": 31
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 32
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
        "value": -8448422598393661124
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ExpressionInline"
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
        "value": 20
      }
    },
    "6": {
      "address": 6,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 2.5
      }
    },
    "7": {
      "address": 7,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.5
      }
    },
    "8": {
      "address": 8,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 2.0
      }
    },
    "9": {
      "address": 9,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 1.00, 0.00)"
        }
      }
    },
    "10": {
      "address": 10,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(1.00, 0.00, 0.00)"
        }
      }
    },
    "11": {
      "address": 11,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 5
      }
    },
    "12": {
      "address": 12,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 30
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
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.Transform, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
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
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "18": {
      "address": 18,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "22": {
      "address": 22,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "23": {
      "address": 23,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "24": {
      "address": 24,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "25": {
      "address": 25,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "26": {
      "address": 26,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "30": {
      "address": 30,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "31": {
      "address": 31,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "32": {
      "address": 32,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "33": {
      "address": 33,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "34": {
      "address": 34,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Subtraction__SystemInt32_SystemInt32__SystemInt32"
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
        "value": "SystemConvert.__ToSingle__SystemInt32__SystemSingle"
      }
    },
    "37": {
      "address": 37,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Multiplication__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "38": {
      "address": 38,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Division__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "39": {
      "address": 39,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Addition__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "40": {
      "address": 40,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Multiply__UnityEngineVector3_SystemSingle__UnityEngineVector3"
      }
    },
    "41": {
      "address": 41,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Addition__UnityEngineVector3_UnityEngineVector3__UnityEngineVector3"
      }
    },
    "42": {
      "address": 42,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "43": {
      "address": 43,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "44": {
      "address": 44,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "45": {
      "address": 45,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_parent__UnityEngineTransform"
      }
    },
    "46": {
      "address": 46,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__get_name__SystemString"
      }
    },
    "47": {
      "address": 47,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__Abs__SystemSingle__SystemSingle"
      }
    },
    "48": {
      "address": 48,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__ToString__SystemString"
      }
    },
    "49": {
      "address": 49,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    }
  }
}
```
