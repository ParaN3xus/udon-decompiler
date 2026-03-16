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

```hex
1f8b08000000000002ffed5c4b5f1b4712ef111292ccd360100f83f15b048c0438ce63370fdb60c2c60607109bc471b00cb24d221e3f1e5eb3b7bdec21a71cf22172cb391f646fded37e817c83ddaa9a1ea9d5d33da3b1d4b286b5f493667aa6bbbaea5fddd555352d45320c5ed6247cadb31576974db11cdb627b6c17ceeec27107de76c9b9fe901dc0f1397ce7e1de241bf36989afebd6007c2f42e9901d41cb63b609c76dbabfca0a70be086d0b50c2abcfe053805a56141ae5d81c5b664b2c65f5f99058a74687fc9a85725909f8bec34ee06e8198c24e32742b055fab70e790eeed00d34ebd47ec3189b503f736a1cd012b02cda7443131106374c2ec433b3fb6c007b91d830fd678c5af47f9b1073e49e11caf1779f91c7c5a795ba7ecd4c3eb17a5763ff37284d34cf0738bd7c7639fd05f1f6ff79a97fb79bd4b42d9a927f6c7db59cca3bf183fa684fe52bcdd1a2f77f0e300a77f59283bf5c57e9df6af3cfa4d4a185da9c428e2601be7e5cfa57643c27982f7f7072f77f2e320af0332461c3d7649f71c5a786ce3c761e1dea0700ff9bccae915257e9cb6e7a56bd8e61a2f9fe1c75e7efdba50c663b7349ed24219f88f452b3189b64b744724ba0ebd5181a751a1ae3866c62bc74c342ad176da5de0f5df13cae298bc22948146cbeb4a1db5fca2d751cbaf1e3aba10504722cf678532de9fe03afcc33ec6a2e6708cd509c7e8eb2a7090c79b8ffcd10f55f3336691d5fe02ec689eedb30c4dd8f100cb8bd3d27f6141d2ad11db16c30a83dd1e8109df07020aeb9ea3b5e388cdb219a5751fb4da05beefd2771e6a6c13d1497717bfbbba40e68af02e084bd3215c5fa0950d17a76db833c5eec3115b3d61d3b0d83c92a8ac43b74558de0a6c0dbef789de1360db5dd34f245cc6e4362b40d35e6577e08a03e83e948bc4e12a7dbf244e0bc4fd225c3ba2357f1756fd3b707c45bdb9653f8073559f6b1c32358f8f035db79774474d73200d729fc1f5f79d46de8a4610fad6493e0da319b2583fb934f10e43dbc8c4c889b5694cdb6e64c5b51996219f6ac585607019dd12aac7b78df72d76d353163532a8ee6192a0ace17ffdf69fa9fcbfef8dc72a459b6519724fcffb88e6052cd24b5bfd2e0ade22c462d12c322ac36fa9e047f7c85a6d20fc76bd6daa175c01e469480ab0d0b5ff0a987948764e5c4071764d09f7363874058ae7d04ade011247f0c6fa2e0d665bb90399562cb17e82e8d41051a921d16835f8db1d2f35244a7295d5802f3582490d827e3ce8106c5121487edd5a0311c47b7b44250f758363884eee848ca11ac03602704011c3fbb1a08330aa80309be421be9baba4c41532a4a11cd351eea89572ab8e7267ad94e33aca5db5524ee82877d74a39a99a0414387d677412b8a314e7ec0eb47d01c2bc24d7f318daaba21aef4981d1d70d0916f59ce8a13931a68cb58271a49b236754085328f9a351847370cd8ec3e6e9ee3639ef659a7b54c6de9cdc9d570b44f501e5028f89176ffc7b4b3297f1cf50ea2443d949939c39991c7ac5636ab5f791dacd33a31b146daa091de7b907976712e77cee52f6b800d45d533cae9fe2edaabe123c67282fbfff85978b78524fbc4335bae9e23f1a6c3fd49991050ed932ac623ff090d6ad5a27345e81e333e202af6f727fdabb2fef9930a09a0994889b0f90df312f05e6f322343586686a4c794c0d1537c127867a620e53efcd858d6e1277aac63e2552fffe56c77e39387946e1894d6f899ef7fc8d74f12368e82547660dea1c401f05aec55a463b46a923b2f5e2a658adef11d2f7a755e8dba4543a0d77a94c67174feb8e2822171254369f237af3d9ad1a42dd6f2797f166311c26c75355c770639ea988a011dc59957abaf953b194621cba5433a6574d8fcee7be54abcfddab63fbb20ecb006c9fd3b17da556b6fb7494afd64ab95f47f95aad94533acad76ba53ca0a39cae95f2a0cee88cd7c9e80ca93a38c31f8cb9c22699708f9ef0b0ceab9da886b0870b7b5e4778b246c223baa978a30e537154477caa0ec42fe89498a95189633aacb3aad0e4a3d2cabdcae6d897b0cce4c17817a475d94e6f4eb10d78db6b32a6500f4b3b53f2dc175b8616f62326ac292e119539aa202abea893675a25cfa7d2d254d9af2dc11e2cba1be4eda28f7142b23e8773e7619937eff593ec924eb2199564333569ea01d53eaab36e2eeb2498554930175802e4bc724ca96314550c513f29afe8a4bca992f2638d94391a5f45e27e9b3b788e8c76f36df06d65aecd48745527d1fb2a89fe5992c8f1be45df5be5d72f966a963dfa1592a20052bea4da535c1ebcbb05148f492ea4ab8b09b07e303eaa892e2a31151fe7c878af039ded80c99b6b3aa46fa990ce3518e94a6c4de2705d87c3072a1cae4a56fc36f0828fea4e24cb769fb87f0ee72f5c5c964392207ca6757c7ea8b368ea3ec595668102ad3cd7dd1ae5a2773df8d55fafd7fc1fd749f9914acadb5549799fc6e36153c8f79e4ebe8f752babcee6ca236f818fbc6a6510690591604227c19f54127c5695866ec39cdde2ebcf5e5d34e437c7e2f0b62c8c95e6a9267a5b0fc97a60e990652803900d902dc4de7700531c1768c59fd2f8f0cf10fdd963dfdcb3ba6da17ab31cd8a252baea7379eeed62712e91430b90a674c86993f413d4aa93a08c64283133f186a3c99f0b8c2d5a5baadd7be9b731c426b0c43d570b530f1b7c6294f7a5f8f622e7f0e55e6e10016777511d368ce2d0b20dc90137f8f673ba784c227ecfd0c6c46a770eb9674517d9a157f45ce180e86dc9b384b26fcdc6f967e2284f6b464ad9d4aa4d64164dede7ffe75630db43a3174018738c46b6977e2d637066f793be1c8ded42252f577b5aeebf969d83c98ad91fafcf9671b705384f16c086f2ae4928b1834df206731453ebf34fc6406c330562540071ce2488193e1e8b5009c7e27169dabda0b0785fb161460ebcf55b6b3658d61cf83da6c0bf20803f6f12fc590efe26b7a64786721a06cdc888292574094ab8771a94900da1123a05252c9854420757c201ede84043e4d0c6dc9231e062a6806302705f9804ae535a04e7799aef846f9709dfd2d72240b7d808574c9cf822c15c45aec3e0f44d9a82b25d80f22f26a16c934661985daf9800da97cd130a6443180a0c0b50de370965bb34fec4079ce11b81ad026c0f4cc2765131020fc85d41e77d176c86fde3d2d366172302c04b2601c610a59a882a7c3152528070d924845d5acfd0d9401f3ebb6809e03d34095e9fcf12234e648341e2982920470420bf6a1620b32104b2570072a559809c092190a30290aba6b39fd52c2d87e640ec3305e21901c435d38bcb1e8dc293d2969835b812e6f825218097330ddea672ffd0526857e63601bcf546641d453bf82eeb682b615050c25f4d2a61942b01778ce153cb603b4a0d423b640ada0e01daaf4d277411d205b8bd07b7717752798c86cfaac605e0be698483a4cb4a86dd653f2b00f96db300194697bd5b00f2914920533e2ebbfc8b886c08a7778f00e677cd04e67408c13c2780f9b899c09c0921987d0298df371398b32104b35f0073a399c0bc1942305302984f9a09ccf74308e6800066be11318f3ef03c6ddbb9864ecf96e42c2ead4f7178244b3b80fd7680a4036c944c7b3cc94f7b3e654dfb447ce92a726de9aa9f97a503a43fd35524aaecfdd267f98840541e50fe240f3de4e958de3b9da1bf4eb815f08705de34fd37366f8adbbea5915c6caacdda2a496bd9b8ddeea1950cfdd9c469967fabf20fd0e2b1144dd11c687e8bff8a70192a6f1153f4b703ff03f2bbefc1d1620000
```
