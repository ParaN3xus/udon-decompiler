<!-- ci: skip-compile -->

```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon.Common.Interfaces;
using Utilities = VRC.SDKBase.Utilities;

namespace QvPen.Udon.UI
{
    [AddComponentMenu("")]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_InteractButton : UdonSharpBehaviour
    {
        [SerializeField]
        private bool canUseEveryone = false;
        [SerializeField]
        private bool canUseInstanceOwner = false;
        [SerializeField]
        private bool canUseOwner = false;
        [SerializeField]
        private bool canUseMaster = false;

        [SerializeField]
        private bool isGlobalEvent = false;
        [SerializeField]
        private bool onlySendToOwner = false;
        [SerializeField]
        private UdonSharpBehaviour udonSharpBehaviour;
        [SerializeField]
        private UdonSharpBehaviour[] udonSharpBehaviours = { };
        [SerializeField]
        private string customEventName = "Unnamed";

        public override void Interact()
        {
            if (!canUseEveryone)
            {
                if (canUseInstanceOwner && !Networking.IsInstanceOwner)
                    return;

                if (canUseMaster && !Networking.IsMaster)
                    return;

                if (canUseOwner && !Networking.IsOwner(gameObject))
                    return;
            }

            if (Utilities.IsValid(udonSharpBehaviour))
            {
                if (!isGlobalEvent)
                    udonSharpBehaviour.SendCustomEvent(customEventName);
                else
                    udonSharpBehaviour.SendCustomNetworkEvent(onlySendToOwner ? NetworkEventTarget.Owner : NetworkEventTarget.All, customEventName);
            }

            if (udonSharpBehaviours.Length > 0)
            {
                if (!isGlobalEvent)
                {
                    foreach (var udonSharpBehaviour in udonSharpBehaviours)
                    {
                        if (Utilities.IsValid(udonSharpBehaviour))
                            udonSharpBehaviour.SendCustomEvent(customEventName);
                    }
                }
                else
                {
                    foreach (var udonSharpBehaviour in udonSharpBehaviours)
                    {
                        if (Utilities.IsValid(udonSharpBehaviour))
                            udonSharpBehaviour.SendCustomNetworkEvent(onlySendToOwner ? NetworkEventTarget.Owner : NetworkEventTarget.All, customEventName);
                    }
                }
            }
        }
    }
}
```

```json
{
  "byteCodeHex": "000000010000000C00000001000000030000000400000020000000050000017800000001000000040000000100000012000000090000000100000012000000040000006C000000010000001400000006000000200000000100000014000000010000001200000006000000210000000100000012000000040000009000000001000000020000000900000008000000020000000100000006000000010000001500000009000000010000001500000004000000DC00000001000000160000000600000022000000010000001600000001000000150000000600000021000000010000001500000004000001000000000100000002000000090000000800000002000000010000000500000001000000170000000900000001000000170000000400000154000000010000000D000000010000001800000006000000230000000100000018000000010000001700000006000000210000000100000017000000040000017800000001000000020000000900000008000000020000000100000009000000010000001200000006000000240000000100000012000000040000026C000000010000000700000004000002400000000100000009000000010000001A00000009000000010000000800000004000001F0000000010000000E0000000100000019000000090000000500000204000000010000000F000000010000001900000009000000010000001A000000010000000B000000010000001B000000090000000100000019000000010000000B0000000600000025000000050000026C0000000100000009000000010000000B000000010000001C00000009000000010000000B0000000600000026000000010000000A000000010000001300000006000000270000000100000013000000010000001000000001000000140000000600000028000000010000001400000004000005040000000100000007000000040000040C000000010000000A000000010000001D00000006000000270000000100000010000000010000001E00000009000000010000001E000000010000001D0000000100000015000000060000002900000001000000150000000400000404000000010000000A000000010000001E000000010000001F000000060000002A000000010000001F00000001000000160000000600000024000000010000001600000004000003DC00000001000000080000000400000394000000010000000E00000001000000190000000900000005000003A8000000010000000F000000010000001900000009000000010000001F000000010000000B000000010000001B000000090000000100000019000000010000000B0000000600000025000000010000001E0000000100000011000000010000001E000000060000002B00000005000002F00000000500000504000000010000000A000000010000001D00000006000000270000000100000010000000010000001E00000009000000010000001E000000010000001D0000000100000015000000060000002900000001000000150000000400000504000000010000000A000000010000001E000000010000001F000000060000002A000000010000001F00000001000000160000000600000024000000010000001600000004000004DC000000010000001F000000010000000B000000010000001B00000009000000010000000B0000000600000026000000010000001E0000000100000011000000010000001E000000060000002B00000005000004380000000100000002000000090000000800000002",
  "byteCodeLength": 1304,
  "symbols": {
    "__intnl_SystemString_1": {
      "name": "__intnl_SystemString_1",
      "type": "System.String",
      "address": 28
    },
    "canUseInstanceOwner": {
      "name": "canUseInstanceOwner",
      "type": "System.Boolean",
      "address": 4
    },
    "__lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 31
    },
    "__const_VRCUdonCommonInterfacesNetworkEventTarget_1": {
      "name": "__const_VRCUdonCommonInterfacesNetworkEventTarget_1",
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "address": 15
    },
    "__const_VRCUdonCommonInterfacesNetworkEventTarget_0": {
      "name": "__const_VRCUdonCommonInterfacesNetworkEventTarget_0",
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "address": 14
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "canUseEveryone": {
      "name": "canUseEveryone",
      "type": "System.Boolean",
      "address": 3
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 12
    },
    "canUseOwner": {
      "name": "canUseOwner",
      "type": "System.Boolean",
      "address": 5
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 27
    },
    "canUseMaster": {
      "name": "canUseMaster",
      "type": "System.Boolean",
      "address": 6
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "udonSharpBehaviour": {
      "name": "udonSharpBehaviour",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 9
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 29
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 19
    },
    "__intnl_SystemInt32_2": {
      "name": "__intnl_SystemInt32_2",
      "type": "System.Int32",
      "address": 30
    },
    "udonSharpBehaviours": {
      "name": "udonSharpBehaviours",
      "type": "UnityEngine.Component[]",
      "address": 10
    },
    "onlySendToOwner": {
      "name": "onlySendToOwner",
      "type": "System.Boolean",
      "address": 8
    },
    "customEventName": {
      "name": "customEventName",
      "type": "System.String",
      "address": 11
    },
    "__intnl_VRCUdonCommonInterfacesNetworkEventTarget_0": {
      "name": "__intnl_VRCUdonCommonInterfacesNetworkEventTarget_0",
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "address": 25
    },
    "__this_UnityEngineGameObject_0": {
      "name": "__this_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 13
    },
    "isGlobalEvent": {
      "name": "isGlobalEvent",
      "type": "System.Boolean",
      "address": 7
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 17
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 16
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 18
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 20
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 21
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 22
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 23
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 24
    },
    "__intnl_VRCUdonUdonBehaviour_0": {
      "name": "__intnl_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 26
    }
  },
  "entryPoints": [
    {
      "name": "_interact",
      "address": 0
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 2974308013393031125
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.Udon.UI.QvPen_InteractButton"
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "4": {
      "address": 4,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "5": {
      "address": 5,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "6": {
      "address": 6,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "7": {
      "address": 7,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "10": {
      "address": 10,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": []
      }
    },
    "11": {
      "address": 11,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Unnamed"
      }
    },
    "12": {
      "address": 12,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "13": {
      "address": 13,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.GameObject, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "14": {
      "address": 14,
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "15": {
      "address": 15,
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
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
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
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
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "24": {
      "address": 24,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "25": {
      "address": 25,
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "26": {
      "address": 26,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "27": {
      "address": 27,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "28": {
      "address": 28,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "29": {
      "address": 29,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "30": {
      "address": 30,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "31": {
      "address": 31,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "32": {
      "address": 32,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__get_IsInstanceOwner__SystemBoolean"
      }
    },
    "33": {
      "address": 33,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_UnaryNegation__SystemBoolean__SystemBoolean"
      }
    },
    "34": {
      "address": 34,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__get_IsMaster__SystemBoolean"
      }
    },
    "35": {
      "address": 35,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__IsOwner__UnityEngineGameObject__SystemBoolean"
      }
    },
    "36": {
      "address": 36,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseUtilities.__IsValid__SystemObject__SystemBoolean"
      }
    },
    "37": {
      "address": 37,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomNetworkEvent__VRCUdonCommonInterfacesNetworkEventTarget_SystemString__SystemVoid"
      }
    },
    "38": {
      "address": 38,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEvent__SystemString__SystemVoid"
      }
    },
    "39": {
      "address": 39,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__get_Length__SystemInt32"
      }
    },
    "40": {
      "address": 40,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "41": {
      "address": 41,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "42": {
      "address": 42,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Get__SystemInt32__SystemObject"
      }
    },
    "43": {
      "address": 43,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    }
  }
}
```
