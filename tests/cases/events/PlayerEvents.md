```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class PlayerEvents : UdonSharpBehaviour
{
    public int joinCount;
    public int leaveCount;

    public override void OnPlayerJoined(VRCPlayerApi player)
    {
        joinCount += 1;
    }

    public override void OnPlayerLeft(VRCPlayerApi player)
    {
        leaveCount += 1;
    }
}
```

```json
{
  "byteCodeHex": "000000010000000600000001000000030000000100000007000000010000000300000006000000090000000100000002000000090000000800000002000000010000000600000001000000040000000100000007000000010000000400000006000000090000000100000002000000090000000800000002",
  "byteCodeLength": 120,
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
    "joinCount": {
      "name": "joinCount",
      "type": "System.Int32",
      "address": 3
    },
    "leaveCount": {
      "name": "leaveCount",
      "type": "System.Int32",
      "address": 4
    },
    "onPlayerJoinedPlayer": {
      "name": "onPlayerJoinedPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 5
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 6
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 7
    },
    "onPlayerLeftPlayer": {
      "name": "onPlayerLeftPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 8
    }
  },
  "entryPoints": [
    {
      "name": "_onPlayerJoined",
      "address": 0
    },
    {
      "name": "_onPlayerLeft",
      "address": 60
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 6990539851667711519
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "PlayerEvents"
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
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "6": {
      "address": 6,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
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
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "9": {
      "address": 9,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    }
  }
}
```
