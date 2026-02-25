```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class IfElse : UdonSharpBehaviour
{
    public void Start()
    {
        int score = 75;
        string result;

        if (score >= 60)
        {
            result = "Pass";
            Debug.Log("Good job");
        }
        else
        {
            result = "Fail";
            Debug.Log("Try again");
        }

        Debug.Log(result);
    }
}
```

```json
{
  "byteCodeHex": "00000001000000030000000100000004000000010000000A00000009000000010000000A0000000100000005000000010000000C000000060000000D000000010000000C00000004000000780000000100000006000000010000000B000000090000000100000007000000060000000E000000050000009C0000000100000008000000010000000B000000090000000100000009000000060000000E000000010000000B000000060000000E0000000100000002000000090000000800000002",
  "byteCodeLength": 192,
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
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 7
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 8
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 9
    },
    "__lcl_score_SystemInt32_0": {
      "name": "__lcl_score_SystemInt32_0",
      "type": "System.Int32",
      "address": 10
    },
    "__lcl_result_SystemString_0": {
      "name": "__lcl_result_SystemString_0",
      "type": "System.String",
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
        "value": -482339120511430956
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "IfElse"
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
        "value": 75
      }
    },
    "5": {
      "address": 5,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 60
      }
    },
    "6": {
      "address": 6,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Pass"
      }
    },
    "7": {
      "address": 7,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Good job"
      }
    },
    "8": {
      "address": 8,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Fail"
      }
    },
    "9": {
      "address": 9,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Try again"
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
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
        "value": "SystemInt32.__op_GreaterThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "14": {
      "address": 14,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    }
  }
}
```
