<!-- ci: skip-compile -->

```csharp
// This test case is basically the same with InternalCall.md,
// except the public entry of function `fibonacci` is manually
// replaced with two NOPs

using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class HiddenInternalFunction : UdonSharpBehaviour
{
    void Start()
    {
        Debug.Log(fibonacci(10).ToString());
    }

    int fibonacci(int n)
    {
        if (n <= 2)
        {
            return 1;
        }
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}
```

```json
{
  "byteCodeHex": "00000001000000030000000100000004000000010000000700000001000000060000000900000005000000700000000100000005000000010000000C0000000600000011000000010000000C00000006000000120000000100000002000000090000000800000002000000000000000000000001000000060000000100000008000000010000000D0000000600000013000000010000000D00000004000000C800000001000000090000000100000005000000090000000100000002000000090000000800000002000000010000000A00000001000000060000000100000009000000010000000E0000000600000014000000010000000E0000000100000006000000090000000500000070000000010000000B00000001000000060000000100000008000000010000000F0000000600000014000000010000000F00000001000000060000000900000001000000050000000100000010000000090000000500000070000000010000001000000001000000050000000100000005000000060000001500000001000000020000000900000008000000020000000100000002000000090000000800000002",
  "byteCodeLength": 428,
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
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 4
    },
    "__0___0_fibonacci__ret": {
      "name": "__0___0_fibonacci__ret",
      "type": "System.Int32",
      "address": 5
    },
    "__0_n__param": {
      "name": "__0_n__param",
      "type": "System.Int32",
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
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 10
    },
    "__gintnl_SystemUInt32_2": {
      "name": "__gintnl_SystemUInt32_2",
      "type": "System.UInt32",
      "address": 11
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 12
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 13
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 14
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 15
    },
    "__intnl_SystemInt32_2": {
      "name": "__intnl_SystemInt32_2",
      "type": "System.Int32",
      "address": 16
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
        "value": 5770255750407469320
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "HiddenInternalFunction"
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
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 44
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
        "value": 0
      }
    },
    "7": {
      "address": 7,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 10
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
        "value": 1
      }
    },
    "10": {
      "address": 10,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 268
      }
    },
    "11": {
      "address": 11,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 356
      }
    },
    "12": {
      "address": 12,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__ToString__SystemString"
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    },
    "19": {
      "address": 19,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Subtraction__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "21": {
      "address": 21,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    }
  }
}
```
