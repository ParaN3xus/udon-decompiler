```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class SwitchCaseShort : UdonSharpBehaviour
{
    public void Run(int mode)
    {
        switch (mode)
        {
            case 0:
                Debug.Log("Mode Zero");
                break;
            case 1:
                Debug.Log("Mode One");
                break;
            default:
                Debug.Log("Default Mode");
                break;
        }
    }
}
```

```json
{
  "byteCodeHex": "000000010000000400000001000000030000000100000005000000010000000A000000060000000C000000010000000A00000004000000500000000100000006000000060000000D00000005000000C000000001000000030000000100000007000000010000000B000000060000000C000000010000000B00000004000000B80000000100000008000000060000000D00000005000000C000000005000000B80000000100000009000000060000000D00000005000000C000000005000000A00000000100000002000000090000000800000002",
  "byteCodeLength": 212,
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
    "__0_mode__param": {
      "name": "__0_mode__param",
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
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 6
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 7
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 8
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
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
      "name": "__0_Run",
      "address": 0
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 8442549199859465148
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SwitchCaseShort"
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
        "value": 0
      }
    },
    "6": {
      "address": 6,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Mode Zero"
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Mode One"
      }
    },
    "9": {
      "address": 9,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Default Mode"
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
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    }
  }
}
```
