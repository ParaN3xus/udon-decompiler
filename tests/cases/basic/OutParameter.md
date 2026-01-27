```csharp
using UnityEngine;
using UdonSharp;
using VRC.SDK3.Data;

public class OutParameter : UdonSharpBehaviour
{
    void Start()
    {
        DataDictionary myDict = new DataDictionary();

        myDict.Add("TargetKey", 42);

        DataToken outResult;

        bool success = myDict.TryGetValue("TargetKey", out outResult);

        if (success)
        {
            Debug.LogFormat("Success! Value is: {0}", outResult.Int);
        }
        else
        {
            Debug.Log("Key not found.");
        }
    }
}
```

```json
{
  "byteCodeHex": "0000000100000003000000010000000A00000006000000110000000100000004000000010000000B00000006000000120000000100000005000000010000000C0000000600000013000000010000000A000000010000000B000000010000000C00000006000000140000000100000004000000010000000F0000000600000012000000010000000A000000010000000F000000010000000D000000010000000E0000000600000015000000010000000E0000000400000110000000010000000D0000000100000010000000060000001600000001000000070000000100000008000000010000001000000006000000170000000100000006000000010000000700000006000000180000000500000120000000010000000900000006000000190000000100000002000000090000000800000002",
  "byteCodeLength": 308,
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
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 4
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 5
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 6
    },
    "__gintnl_SystemObjectArray_0": {
      "name": "__gintnl_SystemObjectArray_0",
      "type": "System.Object[]",
      "address": 7
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 8
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 9
    },
    "__lcl_myDict_VRCSDK3DataDataDictionary_0": {
      "name": "__lcl_myDict_VRCSDK3DataDataDictionary_0",
      "type": "VRC.SDK3.Data.DataDictionary",
      "address": 10
    },
    "__intnl_VRCSDK3DataDataToken_0": {
      "name": "__intnl_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 11
    },
    "__intnl_VRCSDK3DataDataToken_1": {
      "name": "__intnl_VRCSDK3DataDataToken_1",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 12
    },
    "__lcl_outResult_VRCSDK3DataDataToken_0": {
      "name": "__lcl_outResult_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 13
    },
    "__lcl_success_SystemBoolean_0": {
      "name": "__lcl_success_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 14
    },
    "__intnl_VRCSDK3DataDataToken_2": {
      "name": "__intnl_VRCSDK3DataDataToken_2",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 15
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
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
        "value": -4992566944655438277
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "OutParameter"
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "TargetKey"
      }
    },
    "5": {
      "address": 5,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 42
      }
    },
    "6": {
      "address": 6,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Success! Value is: {0}"
      }
    },
    "7": {
      "address": 7,
      "type": "System.Object[]",
      "value": {
        "isSerializable": true,
        "value": [null]
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Key not found."
      }
    },
    "10": {
      "address": 10,
      "type": "VRC.SDK3.Data.DataDictionary",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "11": {
      "address": 11,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "12": {
      "address": 12,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "13": {
      "address": 13,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
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
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
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
        "value": "VRCSDK3DataDataDictionary.__ctor____VRCSDK3DataDataDictionary"
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__op_Implicit__SystemString__VRCSDK3DataDataToken"
      }
    },
    "19": {
      "address": 19,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__op_Implicit__SystemInt32__VRCSDK3DataDataToken"
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__Add__VRCSDK3DataDataToken_VRCSDK3DataDataToken__SystemVoid"
      }
    },
    "21": {
      "address": 21,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataDictionary.__TryGetValue__VRCSDK3DataDataToken_VRCSDK3DataDataTokenRef__SystemBoolean"
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__get_Int__SystemInt32"
      }
    },
    "23": {
      "address": 23,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Set__SystemInt32_SystemObject__SystemVoid"
      }
    },
    "24": {
      "address": 24,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__LogFormat__SystemString_SystemObjectArray__SystemVoid"
      }
    },
    "25": {
      "address": 25,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    }
  }
}
```
