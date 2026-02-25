```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class InternalCall : UdonSharpBehaviour
{
    private int _value;

    public int Value
    {
        get { return _value; }
    }

    void Start()
    {
        Debug.Log(fibonacci(10).ToString());
        Debug.Log(Value.ToString());
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
  "byteCodeHex": "000000010000000500000001000000030000000100000004000000090000000100000002000000090000000800000002000000010000000200000009000000080000000200000001000000050000000100000006000000010000000900000001000000080000000900000005000000EC0000000100000007000000010000000F0000000600000015000000010000000F0000000600000016000000010000000A000000050000000800000001000000040000000100000010000000060000001500000001000000100000000600000016000000010000000200000009000000080000000200000001000000050000000100000008000000010000000B0000000100000011000000060000001700000001000000110000000400000144000000010000000C0000000100000007000000090000000100000002000000090000000800000002000000010000000D0000000100000008000000010000000C00000001000000120000000600000018000000010000001200000001000000080000000900000005000000EC000000010000000E0000000100000008000000010000000B000000010000001300000006000000180000000100000013000000010000000800000009000000010000000700000001000000140000000900000005000000EC000000010000001400000001000000070000000100000007000000060000001900000001000000020000000900000008000000020000000100000002000000090000000800000002",
  "byteCodeLength": 552,
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
    "_value": {
      "name": "_value",
      "type": "System.Int32",
      "address": 3
    },
    "__0_get_Value__ret": {
      "name": "__0_get_Value__ret",
      "type": "System.Int32",
      "address": 4
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 5
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 6
    },
    "__0___0_fibonacci__ret": {
      "name": "__0___0_fibonacci__ret",
      "type": "System.Int32",
      "address": 7
    },
    "__0_n__param": {
      "name": "__0_n__param",
      "type": "System.Int32",
      "address": 8
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 9
    },
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 10
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 11
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 12
    },
    "__gintnl_SystemUInt32_2": {
      "name": "__gintnl_SystemUInt32_2",
      "type": "System.UInt32",
      "address": 13
    },
    "__gintnl_SystemUInt32_3": {
      "name": "__gintnl_SystemUInt32_3",
      "type": "System.UInt32",
      "address": 14
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 15
    },
    "__intnl_SystemString_1": {
      "name": "__intnl_SystemString_1",
      "type": "System.String",
      "address": 16
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 17
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 18
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 19
    },
    "__intnl_SystemInt32_2": {
      "name": "__intnl_SystemInt32_2",
      "type": "System.Int32",
      "address": 20
    }
  },
  "entryPoints": [
    {
      "name": "get_Value",
      "address": 0
    },
    {
      "name": "_start",
      "address": 68
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
        "value": "InternalCall"
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
        "value": 0
      }
    },
    "4": {
      "address": 4,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
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
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 112
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
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 168
      }
    },
    "11": {
      "address": 11,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "12": {
      "address": 12,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "13": {
      "address": 13,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 392
      }
    },
    "14": {
      "address": 14,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 480
      }
    },
    "15": {
      "address": 15,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "16": {
      "address": 16,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__ToString__SystemString"
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    },
    "23": {
      "address": 23,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "24": {
      "address": 24,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Subtraction__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "25": {
      "address": 25,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    }
  }
}
```
