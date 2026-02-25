```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class SwitchCaseLong : UdonSharpBehaviour
{
    public void Run(int id)
    {
        string itemName = "";

        switch (id)
        {
            case 101: itemName = "Apple"; break;
            case 102: itemName = "Banana"; break;
            case 103: itemName = "Cherry"; break;
            case 104: itemName = "Date"; break;
            case 105: itemName = "Elderberry"; break;
            case 106: itemName = "Fig"; break;
            case 107: itemName = "Grape"; break;
            case 108: itemName = "Honeydew"; break;
            case 109: itemName = "Pear"; break;
            case 110: itemName = "Peach"; break;
            default: itemName = "Unknown"; break;
        }

        Debug.Log(itemName);
    }
}
```

```json
{
  "byteCodeHex": "000000010000000400000001000000050000000100000014000000090000000100000003000000010000000600000001000000150000000600000018000000010000001500000004000001BC0000000100000003000000010000000700000001000000160000000600000019000000010000001600000004000001BC000000010000000800000001000000030000000100000017000000060000001A0000000800000017000000010000000900000001000000140000000900000005000001D8000000010000000A00000001000000140000000900000005000001D8000000010000000B00000001000000140000000900000005000001D8000000010000000C00000001000000140000000900000005000001D8000000010000000D00000001000000140000000900000005000001D8000000010000000E00000001000000140000000900000005000001D8000000010000000F00000001000000140000000900000005000001D8000000010000001000000001000000140000000900000005000001D8000000010000001100000001000000140000000900000005000001D8000000010000001200000001000000140000000900000005000001D8000000010000001300000001000000140000000900000005000001D80000000100000014000000060000001B0000000100000002000000090000000800000002",
  "byteCodeLength": 508,
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
    "__0_id__param": {
      "name": "__0_id__param",
      "type": "System.Int32",
      "address": 3
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 4
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 5
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 6
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 7
    },
    "__gintnl_SystemUInt32Array_0": {
      "name": "__gintnl_SystemUInt32Array_0",
      "type": "System.UInt32[]",
      "address": 8
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 9
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 10
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 11
    },
    "__const_SystemString_4": {
      "name": "__const_SystemString_4",
      "type": "System.String",
      "address": 12
    },
    "__const_SystemString_5": {
      "name": "__const_SystemString_5",
      "type": "System.String",
      "address": 13
    },
    "__const_SystemString_6": {
      "name": "__const_SystemString_6",
      "type": "System.String",
      "address": 14
    },
    "__const_SystemString_7": {
      "name": "__const_SystemString_7",
      "type": "System.String",
      "address": 15
    },
    "__const_SystemString_8": {
      "name": "__const_SystemString_8",
      "type": "System.String",
      "address": 16
    },
    "__const_SystemString_9": {
      "name": "__const_SystemString_9",
      "type": "System.String",
      "address": 17
    },
    "__const_SystemString_10": {
      "name": "__const_SystemString_10",
      "type": "System.String",
      "address": 18
    },
    "__const_SystemString_11": {
      "name": "__const_SystemString_11",
      "type": "System.String",
      "address": 19
    },
    "__lcl_itemName_SystemString_0": {
      "name": "__lcl_itemName_SystemString_0",
      "type": "System.String",
      "address": 20
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 21
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 22
    },
    "__intnl_SystemUInt32_0": {
      "name": "__intnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 23
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
        "value": -103067344358545050
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SwitchCaseLong"
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
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": ""
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
        "value": 110
      }
    },
    "8": {
      "address": 8,
      "type": "System.UInt32[]",
      "value": {
        "isSerializable": true,
        "value": [
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          444,
          164,
          192,
          220,
          248,
          276,
          304,
          332,
          360,
          388,
          416
        ]
      }
    },
    "9": {
      "address": 9,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Apple"
      }
    },
    "10": {
      "address": 10,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Banana"
      }
    },
    "11": {
      "address": 11,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Cherry"
      }
    },
    "12": {
      "address": 12,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Date"
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Elderberry"
      }
    },
    "14": {
      "address": 14,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Fig"
      }
    },
    "15": {
      "address": 15,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Grape"
      }
    },
    "16": {
      "address": 16,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Honeydew"
      }
    },
    "17": {
      "address": 17,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Pear"
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Peach"
      }
    },
    "19": {
      "address": 19,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Unknown"
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "21": {
      "address": 21,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "22": {
      "address": 22,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "23": {
      "address": 23,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "24": {
      "address": 24,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "25": {
      "address": 25,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "26": {
      "address": 26,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemUInt32Array.__Get__SystemInt32__SystemUInt32"
      }
    },
    "27": {
      "address": 27,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject__SystemVoid"
      }
    }
  }
}
```
