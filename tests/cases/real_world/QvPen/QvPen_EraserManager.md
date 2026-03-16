<!-- ci: skip-compile -->

```csharp
using TMPro;
using UdonSharp;
using UnityEngine;
using UnityEngine.UI;
using VRC.SDKBase;
using VRC.Udon.Common;
using VRC.Udon.Common.Interfaces;

#pragma warning disable IDE0044
#pragma warning disable IDE0090, IDE1006

namespace QvPen.UdonScript
{
    [AddComponentMenu("")]
    [DefaultExecutionOrder(20)]
    [UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
    public class QvPen_EraserManager : UdonSharpBehaviour
    {
        [SerializeField]
        private QvPen_Eraser eraser;

        // Layer 9 : Player
        public int inkColliderLayer = 9;

        [SerializeField]
        private GameObject respawnButton;
        [SerializeField]
        private GameObject inUseUI;

        [SerializeField]
        private Text textInUse;
        [SerializeField]
        private TextMeshPro textInUseTMP;
        [SerializeField]
        private TextMeshProUGUI textInUseTMPU;

        private void Start() => eraser._Init(this);

        public override void OnPlayerJoined(VRCPlayerApi player)
        {
            if (Networking.LocalPlayer.IsOwner(eraser.gameObject) && eraser.IsUser)
                SendCustomNetworkEvent(NetworkEventTarget.All, nameof(StartUsing));
        }

        public override void OnPlayerLeft(VRCPlayerApi player)
        {
            if (Networking.IsOwner(eraser.gameObject) && !eraser.IsUser)
                eraser.OnDrop();
        }

        public void StartUsing()
        {
            eraser.isPickedUp = true;

            respawnButton.SetActive(false);
            inUseUI.SetActive(true);

            var owner = Networking.GetOwner(eraser.gameObject);

            var text = owner != null ? owner.displayName : "Occupied";

            if (Utilities.IsValid(textInUse))
                textInUse.text = text;

            if (Utilities.IsValid(textInUseTMP))
                textInUseTMP.text = text;

            if (Utilities.IsValid(textInUseTMPU))
                textInUseTMPU.text = text;
        }

        public void EndUsing()
        {
            eraser.isPickedUp = false;

            respawnButton.SetActive(true);
            inUseUI.SetActive(false);

            if (Utilities.IsValid(textInUse))
                textInUse.text = string.Empty;

            if (Utilities.IsValid(textInUseTMP))
                textInUseTMP.text = string.Empty;

            if (Utilities.IsValid(textInUseTMPU))
                textInUseTMPU.text = string.Empty;
        }

        public void ResetEraser() => eraser._Respawn();

        public void Respawn() => eraser._Respawn();

        #region Network

        public bool _TakeOwnership()
        {
            if (Networking.IsOwner(gameObject))
            {
                _ClearSyncBuffer();
                return true;
            }
            else
            {
                Networking.SetOwner(Networking.LocalPlayer, gameObject);
                return Networking.IsOwner(gameObject);
            }
        }

        private bool _isNetworkSettled = false;
        private bool isNetworkSettled
            => _isNetworkSettled || (_isNetworkSettled = Networking.IsNetworkSettled);

        [UdonSynced]
        private Vector3[] _syncedData;
        private Vector3[] syncedData
        {
            get => _syncedData;
            set
            {
                if (!isNetworkSettled)
                    return;

                _syncedData = value;

                RequestSendPackage();

                eraser._UnpackData(_syncedData);
            }
        }

        private bool isInUseSyncBuffer = false;
        private void RequestSendPackage()
        {
            if (VRCPlayerApi.GetPlayerCount() > 1 && Networking.IsOwner(gameObject) && !isInUseSyncBuffer)
            {
                isInUseSyncBuffer = true;
                RequestSerialization();
            }
        }

        public void _SendData(Vector3[] data)
        {
            if (!isInUseSyncBuffer)
                syncedData = data;
        }

        public override void OnPreSerialization()
            => _syncedData = syncedData;

        public override void OnDeserialization()
            => syncedData = _syncedData;

        public override void OnPostSerialization(SerializationResult result)
        {
            isInUseSyncBuffer = false;

            if (result.success)
                eraser.ExecuteEraseInk();
        }

        public void _ClearSyncBuffer()
        {
            syncedData = new Vector3[] { };
            isInUseSyncBuffer = false;
        }

        #endregion
    }
}
```

```hex
1f8b08000000000002ffed5d797814c975ef1148a391007109042c0b2cb03b2ca0195d5c0b2ce840887b752dbbc00a210d6240d74a82359b4d6227cec6b97c27d95cf6263e133b879d8dbdb613db39ec388913e78e736f1c277112e7de1c7fe4fb92aa57d59a3735f5baa7d553a32e6cf4a1527757bf7aeff75ebd7a757645ca61ff627bd8af41a7d7e9701a9d0167d4997226d95f1d2c9d603fe2cabd7fc19961e918fb3dcc9eed71b6fabcc9ff3d146b60bf7bd8d5ac33c7debced8cb0340bcffb9c0cfbbb87bd9b6157fcee75f63fc372c596b297069c4ee7bc73ce591fabf72131082fcdca7b312e57ac9afd6e77eeb2a719608a17928247ebd9af3ef664169e4d30a6dd7c979c2b20d6047b36c2de9971c619cd6b40b17a47ad037f3822592ed32532ad93e90af6bf8afdbfa83c5f29ef3f21af2bd8ff04a72bffc6345b65de27151a6df2fe2594cfbdcfd316f9fc32ba6648c6920a1d37dd2779e07faf56f873f3ac91e901f9fc0abae6e97e44633f2a37a1f0e0e2b156a6f5325d27e93e55042e2e4f87141c0ec9f4a0bc3f84ae59d9157b08f91f0920ff1145fe23323d8c681c56f8b8aaf07183e0e328a2d110c04edcf737ca7483627b9532dd24ef0fcbeb2a25ffb042ef9882ef31993e2aef5f43d73cbd4fa6edf2f908ba66722fe957f21f97f946058f4b5c5c36a3e72e1e719976c87732e89ad37ebf92cfa57d5d5e57cbb45379bf53beff15259ffbfe98bc76f9e852deef12ef2f4d2af9dcf76f2c407f9b08fd6d20f4a7ead5c5e084c2eb09c9eb97e575b77c9e55deeb26b03ba9d03b29e855bafea747a157addc57b13ca5d03b25e9bd5e5e9f56e82594fb41b075d33388cefd0ba8636e7a768174b6caf49ce2a3ce09d9ab5e90d7db84de6b3e8d74cfd32da85cb51c96bfeaabf2d979a5ed388fcae7f76f2afc6c51f8d1c9e025578d4c2f20fe2e0899e2ad82b7f84594979775cbe75d9e3ee0212f2ebf56a6db8bccbf53f2548dcae1bcbe22efbfea4163072ad32deb41f15ef56b157e1e43795cdb7908e5d1c508c900f6d42ff38ea36b9eee92699f7c3e81ae999cd55f54f4dfa7e8bf4fa6bd88ff5ef16ec26dbb97a1fbb89d73f39d546c7719a2f5b07c67b2081997497acf098c132fcafb6e7bbe1be92521f5f7721174f7cabc6f40b6a3ead5ebfd4645d741cbdfa4c12525d30189cf14ba6618d4bc9bf0478388463a80fd34c9b459a68fcb77a7d135259b8effbcb22a6310c19f641e7e98514c41e1bb027435dc37fd3b199c74558588cb596f83173bc7c2f969464013e90f403f628e45a5cdda487f436c19e2bb037e0fb31c5920fa7461111f2b28823337ce7e32a89b32cbee77432f877754b2ec49236b90b2f0d655a68a4bec279fca202b769c757532ac62df6645717a5719db8539fd44e25d1af59d5e4653f4b826d81d17d069763d0e1cf6c1ef3bc06906b8ef61f7e6a0ff37c99af37696be064a2b947d86fdad2bb35f42a6e7f14aa0fba27be7aaa99349c3b94f419df8ba461643231cfaaaa765355c9ae27524f69d059af83a86c2c95442702c6834892185bc7bcd4e0a62ffde020483cb5828a1debe05defb9c564f59f4c870756f0209721afec2ddfae7dff2a59d2f55e68bd6e2a4a0fb729f8f685ec0727ac9d8ba020ade2254562e4d734655f8633af8a14bd45746f845be2ce40bae802a6899f21510db0161e81d165267f21ad73e46981735cdc834a21c43ac53cbc7f6664188b3ecaf49f67f0cae0ab498ae921dc7a4a699f513865245854e15d5e55685bfeff15245f5bc5c3955f07f7a041304827e3c50082ed1210831da65a308160669ee5fedecdd1b0c893bd0f2de66efeb823a6f4c794771af82a91ecf5ac073ab36d40cc61185f0521dc2358be3ad1766a13cc45dafa099202d74b9a7b30d6a9f953af4a01f75d3287a03ec9e08e2bbe06916223f1e030e03f5f38ce79b3242e4f250f95d0ccfc294c26de0c41bed1545db6e1d20dd089d3e73dc529aa9d268265d2b672e0a0448280270de09c2719dca610861685154ce5d2b0fdc322caf377c3ca7b7725715addcd5a0dced1eca0dc617a5c66a1ddad0e9bf6a14ed7e666462823027c35978eb86bc9f2f1195cb1bef3545e3bd16f0de012363a5e18c423ca1439cd7fad8e822233ec01c06b79552205f5f34f2eb00f96451c807e390d2408d4e033021db5fc626993f9b022a3c700ede2cf32070b71a38ea216e00881b3413d97e2c5010d6ea20e49e3f965d14273d285b37ce79cbfcc47ce95ae50d459bf346c07aaf87e30ecf2ba59565ba36b94e8ec016da8ada2837d08df2721de56a3997ab765ffe8ffd2b209ea089afd019136f2a63af2b7327443fbaac0ba3dc72a7404d9c237778b197a5d7810b7e7f448e497897e56d7c9be7f1c9619ce20142ac2bc018b97929f8fc660df70c95faaab105aa46b478a6aa529dcee0e37222ac60e8a40166c4869c34fb9f210745780e3e3f20164415d490385d435652dc6cd3719340dc0c41c78b3b9620e5add2d5c835e6a34257577d4ea7739a3d133836cafb1718c5615636a7771c4ac8e937ff0deffaf440d1ce7cfb7c54582ace287b5b4d6978874ec335b04a82af65e3fa9d65b62dec2d888ed75025eed495b806d994ae64fe6c06ee07e161ad8e8755729a5c7579e9d5724e5cfa18b59c2d7439f53a7b86c29f5dd416460c8a707ad761de50d03b07383e03d67a8b850377a49feb07df21d00fdba6f0787bb366d8911e7adc0575e16811dedba45454fd594759f3c354fde15ae325cf810d8bc1e720b6bb5e57e21ab9ccc07700643b4db88112658f4e9438544c0e4d2718e1742021365011e35e4dc4180b12316ea4a468a414c267532f40b5bac56e8d32b504936513254b2a6cf47b9fce7dd4977b9e411dc50bd65d4c17dde83511730c7ee5535573336509cd3a4ba886456823ece736a0c62d218815dc4f95d64295360481e32c8466cf701713a0b42d94cdb586b5b9ad54d3d846358d7c79764d5dd0a6711bd5b3dbafe9d9dda8d208e1d1b37b8082e7405878b6eb28af948ba77dbdef469af00e8af0a190847752403fa201fa73f160403f48113fac21de501d8cf84354953aa2ab527532526c65ff47d96ddeca2ebce793a4ca3eea55b6e8f9f02115b11eeb166b1c052741cadea52bbb462e4c5767c46001b64a7c394dfc61aa7e1fa3eaf7f10585bebb297b6e0f69cf7b2893ebd098dc8b896026b79722dea921fef980c41b29e25d1ae25b6b82114fe96205884ea717b5abd12757ae0d436bfd2c5407b1b64db47e7c64317cb7826f1d58a20e5541a4dc5a44c7a1543cee86a87816b67e09e46679854dc05e94dc1e2f01be430f58754324544ecea9182a4db9c193941bec62b69181288ae7cfadd4e19db35b81dc6013555d7a34d5e57f03569766cac79ed2f85827a08f6da1c28fd361c38f56aaf3772664e7af8d9a563f1b725a7d9f8e7042ee28f1255c4b13de4f817c3e2cc8077494ebe5a60f5f969b68c20729961f0bcbf2214a7dbd21d5f708a5bebe90ea3b4c61d11f168b2394fa0642aaef2885c560482c1ea56af5e3216bf531ca2c2e86348be354bbf04431843d9c7e3b65164f86358b0e8af2a5b0943b29ca97c352eea2285f094bf90445f9a9b094bb29d3180a691a272996af8665b98762793824cba72896af8565f934c5f2484896cf504e6e34a4933b4b11ce84247c8e02f97a5890cf537e792ca45fbe40b17c232ccb8f51ddee6cc86e772fc5f2cdb02cf751946f85a5dc4f05f9e32508f20728b627c2b23d48d594c99035e571ca38a6421ac745ca1b4debba8999f98eaddb19c55d51dd2c5ecf7ccedcfc5d2f743333ccb0ef40ee46187b1327ade41f0133082380bc937c0d063844bedc3002de0692ff044f3da86f0db212b20127099ea0707a5a87d380619c2659de0e56d42c2c0a9b40ef7823141e8727291c667438e4e67ef3d734e4cfec0aee8474ee4a8133ecd90810c22b25780e1d3d6a4d4510c92e5192cdea243b4c2ed72b5c6d942fd9986685923bf65c484db79e2988549729a9e674529d20f445e1db28d70bcd32fe9e915b5d83c9a2da64fe02d320925ea124bdad93f4498335b43bb427a37d5910449ea210b9a343e4f932fb2c6a7547ae8e17cb4731eb44cc7ac5210ae96774487706f68ad1a8615729295f43f97eba5c21175f7c2264e19aba0b088ca1a1706fde4b27d93025d9dd60be5f873f8e6e8ecb0302ee683c80b76c0bb1ca6b9454cfeaa43a17d82a859f0b6397a6daf2114af26fa05a3dcadf623bed01599e662f89491b2ee59067dc596c44ba70cb1da5247d4e27e9be05b5efae071d850557d332d739a953af962c8824194a926fd4497288906400ea97d04e564e81ba1e7450ea6dd4476ba5d2cd754aa26fa274a3af3d7cff92bb5f4948332b353227ef9a8efac72849be5927495bdece2cfd4eabc591e30625c76b75721c2c420e777fd9e2c893a5e4799d4e9e07153f87b9c9afe95dd083d1f9b785d6ee9b14a7dfa2e3f489c06d519fa62d0ae2e9c244540bd1dc2d0a8f6f0de2edfcfbd13d05f1b1406a0e3cda6809bddd3825d1eb7512b52ea825727b56b9a7b95519aa24b9e3038248314149f16d5424af2f13c70dddb0d06558f65bfae16888490f7ee9fba5d2d52425e5f33a29cf18ec19f6ca882a0352d18b624a51e7a628a9bf5d27f5114fa98b5bbe935f17f39717954e9bd3945c6fa06c56efebf2b7991e67bf67a09e0929dc27439ef6198cb25eca38fb89c56a6149d224b43777599d9f025a73acdc14ec774807d815c8f99d601e9ce3db3f3f06e3bfb8ea3b3ccefebb5eb263e016b63ba6472b5df1cbf10a8fbc8b4b895c5a0c69d8ed71af49ca8f8dab5a2eb55a91827d27bb17684dfe5c7c172f6d49b1e747fa1d6c2508b87dafd8f2bce5cbee309bbb74d9b734bf0d1a7b81807b525a090ebf8cc3aae451f63323dde0e584e3c42b2bd25c0bdf6d12aa95005587f4b033408ceff96d676ef1badc056c03602fd7e400fb1e9380ad99df2776010aa79b3c1b607b09d9d91b4dc2568760cbc5a9a764fbc5636e1be0e213f02e5c6f32efc154b8cec0c6fc392ba01a8fe5a07ab349a8c4aad08c1c65c04729da00d30b953998de62122671d8f8ecfc665d1bc07150757bab4970aa65543d8af630db00d0e1a53980de6612206aa3b70d203d579103e9ed26418acf7b22770bac0df0bc823cd0f79a846735da619f912edad6b8e95514377d9f49d0d6a24060ca6734c806e05a519cfefd26815b01c0f1618d5b3050ed0e81f3490a7e06b30d6025ab3858f14a85f8094347ed177b1676e11889bb516e1aeece413caf8c99c0a91251e3fc053ce6912c88919245745b923eb17a926c3b93dab023e911d026354d4cd2d7d2933ee321c9007dda64d15e3c19c875258b1a88c80d81ea872ed37c08f407bec64727d37c74f207390817ddc13cf0b63f64d2dbba07b789c1e84996c97b190fdf53aaf010e6cb04893c5fdc509a4fd214fae35ed41ff9619370aef381337fc6bcc91c94715350f620287f6431a1cc9fc24e9b83b2c91494071094ef28079423d24b7aaf1f4d5b68957508ca779a8472154099f55986609f87ac4100be6812c0f5d216c7c88a3d903707db660ecc8429301b11983f1a25305b2d04732f02f3c7a204e63e0bc16c4260be2b4a60365908e64e04e6bba30466da4230b72130df1325305b2c04730f02f3bd5102b3d942301f4460beaf1c605261bada174f5b18676e4060be3f4a60365908e62604e68f9b5f6c24c621efccafbdcc3fd4d2187c1b4dc1b703c1f713a61785cce40dfeb6c3b17473bad99fd20157670ab84a04dc07a23370d166e1c0450382f28326a1dc00a76b1433a48f577c1b03b4db14a02904e84f9af68959384bd29d8ae00bf8c50c923b97640cbee5a6e05b8ae0fb2993f0b568aab6897df20683a45da694508f94f0d3269590006a62731f67537cccc41860ab4d01164780fdcc624695eafc8e8d63e94904e687a204a68dd3656904e6874d82b955335d166cc7a8415bdd6e0ade5604efcf966367ca0cac0e19874dd18276d6e47c4fa529e0f00add974c02773f69976e3b4f7f66dce87851ad29688f22687fce7448cfa11d6799c6f3b6e9df5b93bbc711a01f59cceea6fda3eccb11941f8dd6ba221b07e0ce21385f36bf0b4b0cc0e9bf2963dff0db6e04dec7a2db043559d8043d82a0fd787406e89a2d6c7c5623283f6112ca2d1a2b0d72884bb38503c9c710b83f1f65709b2c04f71002f717a20c6eda4270db10b89f8c5620d56a6120d587e0fcd4623658f6af2abe0f41f9e9e8ac75b7b1e3d98da0fcc5e8ac756fb2d02a8f20287f293a11e97e0bad723382f2974d9f3c90a18f65b0a01fb40441f52ba687870b27d2c4699b03e6805b670ab80402ee33a6b7497bedd2c4f7ed9b9e588560fc6c7487385a2dacda6711b4bf6a12da6d1a6867607101076f9259eb6d385ef65e1b2aae40007f2e3a2d76ab852df63a04e5af998472994733640cb6b5a660ab46b0fdbae94d7d59d8dbea4256c4697a16f4aa9721007f23bacd4f8b85cdcf1904ede7a3e31d0f5ae81d1f4250fe66b486d29a2dacf417109cbf558e39c9c2253193328b7db61843e07dc12478f53eb658a6e8d1d83a627ce0e66f476714d2c619c8d308cadf317dce1b751a92ddfdf0f508c2df2dc792d674de39f92296cca0a39b86e63b91f6addbd88ec0fc3d9360ee50d6b74da1cfa1dcfb4b5b1f4530ffbee9d32fb3f33d1fceaa7dd3b15508ac3f884e386ee38af59508ca3f2c97aff43a06af0cbed25830be0581f947d1eadbb45908e70082f38ba6cf26d7b7dcf6b5d7b508b43f8e8e6fdc67a16fdc88a0fc1393506ed6c491d922be7d66ab9f7c0001fba7e518a99c83c665f66b60a5da5604ed9f457b075a938561fa7904ef9f47778cbdcdc231f64104ed5f442b78b2f1e89d5308cebf2cf78c79b00fe9197505c602adc711c0af441de0b485003f8600feab728cce53916c994e2e34363adf8c80fc525480b4719a631702f2afcb3158526c1375c0c206ea0402f3cb5102f3a085609e4460fe4d94c0b431746a4160fe6d94c0b471d3f47e04e6df45094c1b577b1c44607e254a60daf89990c308ccbf8f1298366ee96b4760fe4394c0b471e2a30381f98f5102739f8560762230bf1a2530f75b08661702f39fa23bfe99b670fc731f82f69fa33371d762e1c4dd1a04e5bf9463e22e37bf746f1f76f63002f65fa30aac8d47f8ac45c0fe5b5481b5d1abae40c0fe7b74bcea010bbdeafdce3df399db34ff96c27f707388e77d553693f715d762cefc4efa1cf79fd42cb84c122731273d77f4257db69d8bafaeae92dae00b99cec2f290e1f923e4725f604d71738bf188c3ef03a9f843a8de34fd3f8ffa2a073ca6af7fe391fae4ab4ed2309f7f5de6a1951477c9f7b4fcff89f55e91e27e24d65402dbf3e7e8bf3c3e325cbe5583d1d32dff7ef4143037036c86b16dfe2dc97c7a592837b5259a765d3ad9ff3bdfaeb72ed4ae8373f43f26ed5a5013e1a142bbbb08e9f47a5065cee59a067bc87d9044d4f21becefd12290d80c81174ded38bb1a0359b34075c271fff1f652fcac87aa3cc0de159e85af039c912d3d0fcc2bff1fe671a4a10eee0000
```
