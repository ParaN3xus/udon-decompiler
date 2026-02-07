<!-- ci: skip-compile -->

```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;

namespace QvPen.UdonScript.World
{
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_PlayerMods : UdonSharpBehaviour
    {
        [SerializeField]
        private float walkSpeed = 2f;
        [SerializeField]
        private float runSpeed = 4f;
        [SerializeField]
        private float strafeSpeed = 2f;
        [SerializeField]
        private float jumpImpulse = 3f;
        [SerializeField]
        private float gravityStrength = 1f;
        [SerializeField]
        private bool useLegacyLocomotion = false;

        public override void OnPlayerJoined(VRCPlayerApi player)
        {
            if (player.isLocal)
            {
                player.SetRunSpeed(runSpeed);
                player.SetWalkSpeed(walkSpeed);
                player.SetStrafeSpeed(strafeSpeed);
                player.SetJumpImpulse(jumpImpulse);
                player.SetGravityStrength(gravityStrength);
                if (useLegacyLocomotion)
                    player.UseLegacyLocomotion();
            }
        }
    }
}
```

```json
{
  "byteCodeHex": "000000010000000A0000000100000009000000010000000B000000060000000C000000010000000B00000004000000C800000001000000090000000100000004000000060000000D00000001000000090000000100000003000000060000000E00000001000000090000000100000005000000060000000F000000010000000900000001000000060000000600000010000000010000000900000001000000070000000600000011000000010000000800000004000000C8000000010000000900000006000000120000000100000002000000090000000800000002",
  "byteCodeLength": 220,
  "symbols": {
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 10
    },
    "jumpImpulse": {
      "name": "jumpImpulse",
      "type": "System.Single",
      "address": 6
    },
    "onPlayerJoinedPlayer": {
      "name": "onPlayerJoinedPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 9
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "useLegacyLocomotion": {
      "name": "useLegacyLocomotion",
      "type": "System.Boolean",
      "address": 8
    },
    "runSpeed": {
      "name": "runSpeed",
      "type": "System.Single",
      "address": 4
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "walkSpeed": {
      "name": "walkSpeed",
      "type": "System.Single",
      "address": 3
    },
    "strafeSpeed": {
      "name": "strafeSpeed",
      "type": "System.Single",
      "address": 5
    },
    "gravityStrength": {
      "name": "gravityStrength",
      "type": "System.Single",
      "address": 7
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 11
    }
  },
  "entryPoints": [
    {
      "name": "_onPlayerJoined",
      "address": 0
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 1242201996742476998
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.World.QvPen_PlayerMods"
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
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 2.0
      }
    },
    "4": {
      "address": 4,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 4.0
      }
    },
    "5": {
      "address": 5,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 2.0
      }
    },
    "6": {
      "address": 6,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 3.0
      }
    },
    "7": {
      "address": 7,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 1.0
      }
    },
    "8": {
      "address": 8,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "9": {
      "address": 9,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "10": {
      "address": 10,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
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
        "value": "VRCSDKBaseVRCPlayerApi.__get_isLocal__SystemBoolean"
      }
    },
    "13": {
      "address": 13,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__SetRunSpeed__SystemSingle__SystemVoid"
      }
    },
    "14": {
      "address": 14,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__SetWalkSpeed__SystemSingle__SystemVoid"
      }
    },
    "15": {
      "address": 15,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__SetStrafeSpeed__SystemSingle__SystemVoid"
      }
    },
    "16": {
      "address": 16,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__SetJumpImpulse__SystemSingle__SystemVoid"
      }
    },
    "17": {
      "address": 17,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__SetGravityStrength__SystemSingle__SystemVoid"
      }
    },
    "18": {
      "address": 18,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__UseLegacyLocomotion__SystemVoid"
      }
    }
  }
}
```