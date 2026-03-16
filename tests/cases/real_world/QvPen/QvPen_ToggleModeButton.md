<!-- ci: skip-compile -->

```csharp
using UdonSharp;
using UnityEngine;
using Utilities = VRC.SDKBase.Utilities;

#pragma warning disable IDE0044

namespace QvPen.UdonScript.UI
{
    enum QvPen_ToggleModeButton_Mode
    {
        [InspectorName("Nop")]
        Nop,
        [InspectorName("Use Double Click")]
        UseDoubleClick,
        [InspectorName("Enabled Late Sync")]
        EnabledSync,
        [InspectorName("Use Surftrace Mode")]
        UseSurftraceMode
    }

    [AddComponentMenu("")]
    [DefaultExecutionOrder(30)]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_ToggleModeButton : UdonSharpBehaviour
    {
        [SerializeField]
        private QvPen_Settings settings;

        [SerializeField]
        private QvPen_ToggleModeButton_Mode mode = QvPen_ToggleModeButton_Mode.Nop;

        [SerializeField]
        private bool isOn = false;

        [SerializeField]
        private GameObject displayObjectOn;
        [SerializeField]
        private GameObject displayObjectOff;

        private void Start() => UpdateEnabled();

        public override void Interact()
        {
            isOn ^= true;
            UpdateEnabled();
        }

        private void UpdateEnabled()
        {
            if (Utilities.IsValid(displayObjectOn))
                displayObjectOn.SetActive(isOn);
            if (Utilities.IsValid(displayObjectOff))
                displayObjectOff.SetActive(!isOn);

            switch (mode)
            {
                case QvPen_ToggleModeButton_Mode.UseDoubleClick:
                    foreach (var penManager in settings.penManagers)
                    {
                        if (Utilities.IsValid(penManager))
                            penManager._SetUsingDoubleClick(isOn);
                    }
                    break;
                case QvPen_ToggleModeButton_Mode.EnabledSync:
                    foreach (var penManager in settings.penManagers)
                    {
                        if (Utilities.IsValid(penManager))
                            penManager._SetEnabledLateSync(isOn);
                    }
                    break;
                case QvPen_ToggleModeButton_Mode.UseSurftraceMode:
                    foreach (var penManager in settings.penManagers)
                    {
                        if (Utilities.IsValid(penManager))
                            penManager._SetUsingSurftraceMode(isOn);
                    }
                    break;
            }
        }
    }
}
```

```hex
1f8b08000000000002ffed9d4b571bc915804b8024c4c3806dde60830123f39084c0cc643299898db1878c6d3c0891c9781c2c83c0b2411024fb8c7759679de5ac72724eb2cc32d9e5e417cd3f48eebd5d2d4ad555dd6aa416ddcea083aa1f55b7eefd6e3d6e5777434b92c14f6801be76d8165b63099665fbec8415616b0dd263f8187be6f167ec0cd243f8cec1b90536e150127f664343f0bd017b25568692efd81ea4053a9f6179d8de80b279d8c3a307f09b875ca1362894650fd8267bca0643fd0e2276a850891f0ba15da176f8becf3ec0d93c29859524e9d4207c65e04c89ce1d83d266bee7ec0599750ce7f6a0cc193b0299af4862fb3fc38c369891b4f33406bf7086fd91efb7f063ed7c5bcc1be66987b01f81dfdb7cbfd385ac084f87f8f6acb08ff4fe2de533eb8af3fd284f87a5f2c34679729ba8f308cf77472a3f22c96de369174f47f9f939611ff2b4fcc8f75b79dacdd31b3cffbcb08fe94dcec2dcc67482e75de0fb57783a29e49d14f2623ac6cb2c0afba8cf9f25d966b95b3c7f42d8c7745ce236cee51c49dca6045dccb23d924f92d2f95e7e3c25e9d227ece3f9254346e8ef461afeabe483ab92cd73d536b74d5c920fc6251f70766d432e7d3025f960ca90d3fa93e48369850fae39f8e0ba3b1fb42e687cd02fd93c576d73257fb37d3025f980b30bffe8d207d3920fa6b99cb2e48319850f061c7c30e8ce076dff907c50357e864334ab7c05e37c8e9db2240da8775c4c7f6649e7890f45475a8cb9026640acb60c53cc290850cc3e599adbca6c99a595b3cf70a84bd07b8dbe7390a34042d3d62afe65a902953b824f5e983a4b70fc11cdbc387916e04c823d86144bbd04a4cfe1532d6507aa3d82e937cfb6e1fb94e4bd04b5ad399d4cc269562eb305328d28e0188e98404f61ff8834ccd0f77bd2344fda6fc0b132c52445884aee43fa03d566b5fd0cb655756e73646a1d5fb83a6e841ca69b1e8035a87d92faf5cf1eb90c8f20fa489a77c3b6248dc87fb278e26786c62013a620db90b16484b955c7d22c4963f99685a07b1bad16aadbb7c17b95add8daa22683ee1e250bce3dfc97d27fe23f3dfcdb78b8dab46596a43079ccc1343bb0282f1e1ab048b037211c6e4ba1a232fe900a3f85dc9926e237f215289f7b07505420392084e1cf37a0cc331ae7ce27d00c08c6aa4e418cd1d31242be5d408f579e8764d2137e39771f4496e183e52d1e4d45f8454a5c31e53a19a6734b8bca2dedcd768bf33864e796f68a5de76ec11f35c19886a0930e3a82ad2a8214177eef29416bc0666edd87b2af81c47b9a85df4179558067cf142fe81725a66a9e9dc473421976bad34847b84d45b8e37246ee8bb5500c77076b6ea1ddb603afdbf61956d143a785b69b480fcf9d90941ce4754f102f00e765826a7c3d846f48b126e6a4820e614485909683de788a300bc78c6ba2753a5ba0401a43ea1c49df049ddff0801bedd1e537211ad3cc3bd2c41e776fcdddbf8f68276875d33b6d759e892a3c93eae20b451603629201a8bb4670bb4a703b5f6591279bffc28f45784c2f3ca6137e5d211c1bb21be11d2ae19d7cdd46ee42218be41ebde44e9dda030ab5f75daadda512dec1974ce4b133a412dead17dead121ee56bcb96880eed3ca528ed090d1539689cc672bca5cea8bece2b3a83867593810b837a74068da80cc2debc0b9f14fcbeaf1cc623b8ee62dcfc70635aafaef65155ed4342edbbfcfe4696ee6d60af7f4001c12beae56b340eefb1b7aeb4e9d3811e6b40cbb9aa133eae10dee252f8351dc71b765e5c6a9017afeb6abfa9aa7d50e1c575ea1c86eff6d963d8463938a314c18b6e74e9d7619e50606e7589794067e8a41de67483300fea6abfa5aa7dc4b6b36428763ea0802147018479e5e846a321dd0c31a50cb25ccc10c33ac9d3f54a1ed1499ea957f2a84ef2ed7a258fe924cfd62b795c27395eafe41baa50b7bbd9eb0172a8e8ee5ae14ecdc1eb9c662dc0a97e5d387a53458f0281b79772a160ca3ca17daccd7c20a071970af335d35e20da499b4b85c6e8abf3ce846e92596c404436a9139e6880f05b2ae1317e7fd1f132a7532f784a3790a4ea1d48a6759297ea953ca3939cae57f26ddd54bdac9aaa37a42e5dbdbe90a009fc041af32e444627d464f7a8f0b7d4648de95d57dafe9cdd593781c0accede1595bd9f5596f83210b57f0df5e640873c74cd320d5a054a8da10a75dc802da37801baa6acb538c436cea2b8cea2bb2a8b3ed70e45aa358b841092dde33739dff3d0b0766fedc039e4e1c6aa3b3aab5655567d5173bbcc52fc7e06799fc2d9438adf8d1bb7bb4d6a81733acb3e5159764fb2ec7c4154b46a9dfd01b2e7788bfc60d1f6bc94fe78a3ec9bd7d9f7a9cabeef2a3dcc5c12179fc130f4cbd305408e07051b959cebd41631c716b5d73c6f9d679ccd236ab7d50f3cee90f70b956b38d96ef1a6925dff7543644147e4172a223392c7ef813ea8fb076ed5215985236c9eb42cb3d736fe76a3e7a24ecfcf2ede321f93cf4a6c9b6e86142fb5652674f6fd52655f5ab24ff4beec9347dc27b5da70d19694d459f0b9ca82bc877d2b5377dfaa7566bcc8ec91d271fa958a53d6634e45c8bb065595e836f7b150c6cde873110e4b3a0e5fa8387c59538fbe073aecf3c8e7a4213dda69b48ac227444bd4eb941367ef674403f74a2c494bc329174f0162edc7d0e6701cd9aeb457e76702bfb47926f0a0618f873959a16a9109a14d8ad639dba47f142eca2d326501695a06ffd82cfdb5e8d596242daecf5fb035396b81f366a4b5d6e74a9d1e7231043ce5b17b28466ba9850ab61c39c4b116a7c59f4512603e39d5808761b1691903c9190f1030488a865b527df4ca8787882284088be36c150c388ce044c392f0871e3d915aeb2363d621a38706e91f683deb8ce4edcb4308dd5ef29be66be21010b7b491b8a6639dcf4cea1925853c1efc9f4f1aa95e6a1400e1b639c6522f7fe8652f1fe037950c8f1521935dc49992ebafe721d258d57830d798b707ac63c20d61c07cd40c947bbc2d966d83560f5146bd42d92da0fcca4b94b883dd76130a7886a9c72b4c6101d3869798cc1bef87daee9badba9c58f20e66cc2b989d02ccdff809662a80306302ccaf9b01533716ca8bc6a90076f30e01e6632f61ce72984790e9881e36b13e80b62badcce81faff61476a757b06f09b09ff86716bf1bc0597c4040f9d44b9418cfeed33c8e2f9ae12ab0184d6e42807de01dbe3eaff045057c9b5ee2ebe62d112b3ea08e6fcac6e54dcfc085bd02c70470cf2eb30b077f1e6f17507ee39fd1301dc0d1b05740b9e525ca29c595b6fa0903eba34ee7f7d13c6daf0b5e41be2940ce7809795201f98c6ec161e853842b08e34df58f6d406811006ffb67405809e080705d4099f512658f4378540c60701411e0ed780d4f1d1c157996e0b5bc9000efb75ec2c3f0a144a36299172a05f092b055c0f5ad97b8fa1dd6c89bb4c4d6ed15c84901e4effc02321540901302c8effc330b2f05702cec11503ef70fcad500a21c14507eeff55d9b63fed259f03a6f9b80e9453346415d8b0bfa74724500f97bbf800ce274d22580dcf50bc8e50082bc2a807ce91790e90082ec1340e6fc0272258020fb0590af9a7123561772cb37623f0de08dd81901e69e9f6006f1aef6900073df4f3097020873588099f713cc7400618e08300ffc0473398030470598877e82b91240986302ccd77e8279378030c70598053fc15c0d20cc2901e61b3fc1fc248030a705986ffdb326b91cc035c96b1fcf8b3a295c5dc07f84120957de8b51dd608b57ad8fc6ab9e718fd77033385ed3f374c69b367d9c23feadb527a4470e4ae6283d7feb26497f646ed5e51b7cf6329d5f893946542175cf39f2d56b3e2a4beb79e5a7cbc62b49fa937b1fb3fd45feff4b2a2fe90dd20b6159f0fc3efff3809b90799f94c29776c2ff036c1a6175b66b0000
```