<!-- ci: skip-compile -->

```csharp
using TMPro;
using UdonSharp;
using UnityEngine;
using UnityEngine.Animations;
using UnityEngine.UI;
using VRC.SDK3.Data;
using VRC.SDKBase;
using VRC.Udon.Common;
using VRC.Udon.Common.Interfaces;
using Utilities = VRC.SDKBase.Utilities;

#pragma warning disable IDE0044
#pragma warning disable IDE0090, IDE1006

namespace QvPen.UdonScript
{
    [AddComponentMenu("")]
    [DefaultExecutionOrder(20)]
    [UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
    public class QvPen_PenManager : UdonSharpBehaviour
    {
        [SerializeField]
        private QvPen_Pen pen;

        public Gradient colorGradient = new Gradient();

        public float inkWidth = 0.005f;

        // Layer 0 : Default
        // Layer 9 : Player
        public int inkMeshLayer = 0;
        public int inkColliderLayer = 9;

        public Material pcInkMaterial;
        public Material questInkMaterial;

        public LayerMask surftraceMask = ~0;

        [SerializeField]
        private GameObject respawnButton;
        [SerializeField]
        private GameObject clearButton;
        [SerializeField]
        private GameObject inUseUI;

        [SerializeField]
        private Text textInUse;
        [SerializeField]
        private TextMeshPro textInUseTMP;
        [SerializeField]
        private TextMeshProUGUI textInUseTMPU;

        [SerializeField]
        private Shader _roundedTrailShader;
        public Shader roundedTrailShader => _roundedTrailShader;

        [SerializeField]
        private bool allowCallPen = true;
        public bool AllowCallPen => allowCallPen;

        private void Start()
        {
            pen._Init(this);
        }

        public override void OnPlayerJoined(VRCPlayerApi player)
        {
            if (Networking.IsOwner(pen.gameObject) && pen.IsUser)
                SendCustomNetworkEvent(NetworkEventTarget.All, nameof(StartUsing));

            if (player.isLocal)
            {
                if (Utilities.IsValid(clearButton))
                {
                    clearButtonPositionConstraint = clearButton.GetComponent<PositionConstraint>();
                    clearButtonRotationConstraint = clearButton.GetComponent<RotationConstraint>();

                    EnableClearButtonConstraints();
                }
            }
        }

        public override void OnPlayerLeft(VRCPlayerApi player)
        {
            if (Networking.IsOwner(pen.gameObject) && !pen.IsUser)
                pen.OnDrop();
        }

        public void StartUsing()
        {
            pen.isPickedUp = true;

            if (Utilities.IsValid(respawnButton))
                respawnButton.SetActive(false);
            if (Utilities.IsValid(clearButton))
                SetClearButtonActive(false);
            if (Utilities.IsValid(inUseUI))
                inUseUI.SetActive(true);

            var owner = Networking.GetOwner(pen.gameObject);

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
            pen.isPickedUp = false;

            if (Utilities.IsValid(respawnButton))
                respawnButton.SetActive(true);
            if (Utilities.IsValid(clearButton))
                SetClearButtonActive(true);
            if (Utilities.IsValid(inUseUI))
                inUseUI.SetActive(false);

            if (Utilities.IsValid(textInUse))
                textInUse.text = string.Empty;

            if (Utilities.IsValid(textInUseTMP))
                textInUseTMP.text = string.Empty;

            if (Utilities.IsValid(textInUseTMPU))
                textInUseTMPU.text = string.Empty;
        }

        private PositionConstraint clearButtonPositionConstraint;
        private RotationConstraint clearButtonRotationConstraint;

        private void SetClearButtonActive(bool isActive)
        {
            if (Utilities.IsValid(clearButton))
                clearButton.SetActive(isActive);
            else
                return;

            if (!isActive)
                return;

            EnableClearButtonConstraints();
        }

        private void EnableClearButtonConstraints()
        {
            if (Utilities.IsValid(clearButtonPositionConstraint))
                clearButtonPositionConstraint.enabled = true;
            if (Utilities.IsValid(clearButtonRotationConstraint))
                clearButtonRotationConstraint.enabled = true;

            SendCustomEventDelayedSeconds(nameof(_DisableClearButtonConstraints), 2f);
        }

        public void _DisableClearButtonConstraints()
        {
            if (Utilities.IsValid(clearButtonPositionConstraint))
                clearButtonPositionConstraint.enabled = false;
            if (Utilities.IsValid(clearButtonRotationConstraint))
                clearButtonRotationConstraint.enabled = false;
        }

        #region API

        public void _SetWidth(float width)
        {
            inkWidth = width;
            pen._UpdateInkData();
        }

        public void _SetMeshLayer(int layer)
        {
            inkMeshLayer = layer;
            pen._UpdateInkData();
        }

        public void _SetColliderLayer(int layer)
        {
            inkColliderLayer = layer;
            pen._UpdateInkData();
        }

        public void _SetUsingDoubleClick(bool value) => pen._SetUseDoubleClick(value);

        public void _SetEnabledLateSync(bool value) => pen._SetEnabledLateSync(value);

        public void _SetUsingSurftraceMode(bool value) => pen._SetUseSurftraceMode(value);

        public void ResetPen()
        {
            Clear();
            Respawn();
        }

        public void Respawn()
        {
            pen._Respawn();
            SetClearButtonActive(true);
        }

        public void Clear()
        {
            _ClearSyncBuffer();
            pen._Clear();
        }

        public void UndoDraw()
        {
            if (pen.isPickedUp)
                return;

            _TakeOwnership();

            pen._UndoDraw();
        }

        public void EraseOwnInk()
        {
            if (pen.isPickedUp)
                return;

            _TakeOwnership();

            pen._EraseOwnInk();
        }

        #endregion

        #region Callback

        private readonly DataList listenerList = new DataList();

        public void Register(QvPen_PenCallbackListener listener)
        {
            if (!Utilities.IsValid(listener) || listenerList.Contains(listener))
                return;

            listenerList.Add(listener);
        }

        public void OnPenPickup()
        {
            for (int i = 0, n = listenerList.Count; i < n; i++)
            {
                if (!listenerList.TryGetValue(i, TokenType.Reference, out var listerToken))
                    continue;

                var listener = (QvPen_PenCallbackListener)listerToken.Reference;

                if (!Utilities.IsValid(listener))
                    continue;

                listener.OnPenPickup();
            }
        }

        public void OnPenDrop()
        {
            for (int i = 0, n = listenerList.Count; i < n; i++)
            {
                if (!listenerList.TryGetValue(i, TokenType.Reference, out var listerToken))
                    continue;

                var listener = (QvPen_PenCallbackListener)listerToken.Reference;

                if (!Utilities.IsValid(listener))
                    continue;

                listener.OnPenDrop();
            }
        }

        #endregion

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

                pen._UnpackData(_syncedData, QvPen_Pen_Mode.Any);
            }
        }

        [UdonSynced]
        private int inkId;
        public int InkId => inkId;

        public void _IncrementInkId() => inkId++;

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
        {
            if (Networking.IsOwner(gameObject))
                return;

            syncedData = _syncedData;
        }

        public override void OnPostSerialization(SerializationResult result)
        {
            isInUseSyncBuffer = false;

            if (result.success)
                pen._UnpackData(_syncedData, QvPen_Pen_Mode.Any);
            else
                pen._EraseAbandonedInk(_syncedData);
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
1f8b08000000000002ffed9d077c5445fec07703210935104a101010944548b249a8a26220340d10498228608849802585980d01ec5d6cd83b2a7654545454b0f7eed97bafd7efbceeddf9ffdfcdcc9b97fdedecfcdebe97b7b3fb86937cc8cbbe7def37f3fb4efbcd6f5a5a818ffcf38f27bf16fb16f966faf27d55be3adf5a5f33f96b26b936911fe39379bfdcd74aaeabc8ef1af2dd78df88386fd27f63fcb9e4f73cf229ec6b236faef3d5926b887d5fe1ab277fcf23efd6934ff4ee4af2bf9e3ce5ef4a5eaaf295fa16fa16f806f907c411b198bd14e6f7fc542f7f26f93dc3b7917c5bcf22450329605f0d22bf2ac83761f65d1389b4f9dc52df72a65613f9ae96bcd3ea6b24328f6112335f228af8994eec32845ffbf2eb5ee47f16ff3b8dff9dc9ff86cff7e3d7a1369fefc2af7bf3eb30f2bf1bf9ff94f0fd707eff6907321bf93bcf80cff4dac0ef3f0b3e9314f1b708ef9bd726a0cb48211ee633fbf0eb5afefd73e033bd360319cd20dc2c210e5bf9e751fc3a9a5ff7e5729fe79f47f06b0bbfff02f84ce4a499baf4e4d763f9732f82cfb2e75af9732f81cff4ba1fbfe6f0ef5f16de0b0bef85f9750cbff617de0b90ffe924f96e71909eeb84f45cc7af6d427ab619ba752945d2b3dd417a6e10d27303bfae0732d60bf1784588c729483c360219fb77227f8fe7d7714299e9c1afc709697e1c8fcf4fc27379fcb95785343d5e78ff78e3fdae73f9e77cf03ebd16f0389374cdf881dfebc5af2708b24ee0b23609cf8d13e262ea7a9290f627f1eb89fcfe6be033bd06f9f514fefdebe03309373d5378fe64fedc2f8cf8a70fe5f70bc1f7665af5e6d753059d4ee5b29709cf99b2dfe09ffbf0eb69c2fba7f1f7b708cf99efbfc93f67f3ebe9c2fba7f3f73f159e33df7fab13792b0fc95b6708619f6184dd6d89f0dc38246f9d29bc7f267fdfac0f8ac0fbf1f2d65982acb3b8ac6f85e7c47c6ea6cfd9c2fb671bef674ce09f37f1efdf16dedb84a4eb3982bc73b8bc33f8e77305797d84fb623a9f27c83b8fcb7b977f3e5f90972ddcb793ee66ba5c208475811156e614e1b902c092d6e365166114701975fcd90d16cf163b681772f875b310e7cd3cbc6f85e7ccbcf80e6893e8f542e1fd0b8df7b3460bcf99ef9bdccdfc31915f27f1eb64fedc7b0e74b84888c3453c0edb84e7f2101d2e16debf98bfff2be1b93c4107abb899699e0eea3eb10dbb047c37d5411b7600bf76b3907d6927654fe3d70c0bd9977552f681fc7ab944f641e019990d7bb08370a6f3eb1592700e01cfc8c2297110ce0c7ebd5212ce4cf08c2c9c5207e1cc32f251cf1ff9e7d9fcf3f90eda24f37a1588eb1c210e736db41b5661cd339ecdad43c2be1a847da80b9bed1ac1aebc865faf05f2af35ca70af6d16f20f33e2db6f0312df2d40de7c17f1bd4e88ef75fc7a3d907fbd11dfde4b2ce42f8813df1b80bc850ee25bceaf370af5e08dfcba55b0cdb7f2b8fe68c4a74f509073137ffe7dfe7900b80fe57d102daf4fa5455c4dd93723b2cdfb1fdad0f7707ebd05f032e5dccae57c049e31efd3eb6dfcfb8fc16712f7ec1f0439e67b8bf8f5767eddc6dfff047ca6ef971a2cb3cbf8f525e1bd3bf97b9f82cff47a07d0e10e7ebd4b48c7bb7818dbb8ec1d4218e67b15429e3175a8049fe9f79ff1747fcd01ebed12d6770bacb783fbf47a8fc0fa1e438f7ecb0439db05d6f7f2eb0e81f50ee3fdbe5cffbebb78799a20bc77bfc0fa7e7ebd0fe8701fbf3e20b07e80c7b13f973d5408c37caf4a60bd5d60bd3d9a75df2936582fe6d79d429f7e278fd397fcf311423d6dd6f765167e27f27c8ea9c3835cfee7e0b3193ebdff85109f32213e321dacf41ac8af0f81f83d64e89473028fdb56f02c0debcb38efd2eb129b7eb641fc7aa4cde797f238958370485cfb73bf4dff032d641c05c234c35a66bc37a05288cfc312bbe368f08ccceea8e6d7e5c8f72b1cb41bb9fc5a63934bae90c773411e8ff7ee23fcd9afc067286b37fffe6bf099301fb05dc88bbb85bcb89b5f77011d76f1774dfb7f30b80fdb41fedcc0a142391a0c641dc3dff9c6868e83b9bc7223bd07b6f1fb6639ad057984db65fd7d36e4d6f3fcf826c8c7621eb353af3c2ab07bd488efa09e1632560a79d669dcf3244c57f1eb633c3edf82cf343e3f09613eaeb89cd0ba74ae10e613923057c70933e4a0ecad11dad727f9bbdf81cf186f19d3a8b0d2fd6c94652ec93935be165f010b7cac83e120f3cdf803415474b73463ecc4378605dbe6db485ead978dc654b1b19e365fb1af483a1a33d8df13c47b26fb5d439e0831a1dfc706b13b26081ab946f2530f8692c2e4fe1c361245079342e49b7cd2a885d85b2b7c85a4c25f2a48594c826df4ad239f2ac9ef16266f058976ec93f154a2c34ee23b8b884c6354ac89dc3181b690cf8d2c8615ec773b8b693d8bfd3c72af8d8dd13593e233835c37b0d062756f257fcbc2ace4c8e4715ceee8be310467265329d186c6be8095839f5324152942d177fb9e17c3ae05b48cf8cf8d49899f191a954c3a737618320a8d61dfa87b45be02fad1bf2886a0731d633594e76f83f724df044b5de46468720f611a445278d165238a379ff4f6afd3a3552bf615d066c63f348e6a5660a9bc807f608c046b15d2d3bb06694445fc7e197eeabef4572411bff15c883de73c01bab196293a01fc2359f3deee2b67f55ca401ad208269502d444c3e78a29a5fe71311cde4ff2aa64c4cfa05bb71f76e40d2c0c653034b8434592264263b11e2d73a568990d9a1572411e83f39c12c8460bc386004bbc80832eb6c995282b1e699f9d70cf2ee6a42a29db5b9ebc8fb3273ce9a69776a6e0a4ce53c7b309e23a446a6b3186184bbca08d308fa572b255c45ee1906e82cf66d88592dd47ea1b3a5ead8bc261a16d5057bd6e4399f4d575ac762614dbe6787b611f2053dcc795daae244878bd30a58d8931c0453425e6e61497b18b9b3b163b69593a07bc39e44d70236703dc1451c9c864fb2ee29d347b00aa36bb09731cd88dd38657a7a467a5a416fa7500cbb9706d479287da2a1f4710a458c83d3f0bb16649b7d47abd78c203a4b1cfe4a375380aada35981d950ed18fb174317aba4de62c4036652fe800d07cfea6d3b867f3797b460b932eaf13fbb23a516579c5eaca74595dd92ff94695618934c6b1d0e535208dee70a106ecfec1cdd3e4ac73508bc83a0e18c16e328239a9e91574ce1ea2fea241b6eda1019646bd536b2843422f98c3fdf962a4987f2b4b88d400a31a9009cf94254dffd41902d4606fe38e0caa53620d8141b64db0dcb8d58ddb9862c99d254beefedc4519137931a973f1a4ee2e4b6a366e184a495297112d3632391465d8d790e0c4a653cdbb88561f1b03cbb34856f7b1cafd0ff987b4624358b6521b3e96b17ac8d29ff9e8d7a4c8e6af61d21792ba6f0df7682636070cb55ddc87b174c9b7b42edcc7164b999eb2223f885b45718bfc30bcc8f7c2040f7729b8b72c2fb131cdea94e425ea69a01ecc7af2ac75bad027ad73cd08dbb96624cb35a32c728db37861f9a38f8c361b895da1947625c9bdc66aa6880ef3d95babf9fd688db0a7ac79ef639bf728c67b341bea4e4ccc30e2d932e2b441f1d7a5987815a989685e4904f9d1b6c9efcbc8076c917716432c05faca52808df8ae4c490d53c1dc21754c52625ba9fd6ca7c218960afb5bd4376e6289a5443f594ab0791e9549ecbdd1efd63229d4a3efbc0747f3ee387158418e792cc39c2b59ff182f0a18c21c1942dabcfa4f4b49662e61f79b580f2632885c4efe0ab3e78d7b33f9376dccb1128aeb5489956abf10ec6fbb108c63a933dba210a4423b2ce5fbcb527e6fafa5fc22f2571bb897cc941f6f3be5f33a99f26ab5c3527e802ce5a9dded5f9a9441ac0a5fa9ef30822e9f5c6b98fe91bfcce921912123f369ebb4ca970da4b025c4c3c16055e742365782b17f1948ff39c8b280fbc0b0441b28eb300de5eb37c5e623b6c73416ef310d92658791a9f3be2ce67d589ab0c59d1a4fb1ce2a45b68b75715c9f88fbb862e99d8bf95627600e5f07bed5c158669ae83633ed25933c842f1d8cdbaf1f830b1e22139cc9d7228a1302989f4b149e850b1f8af198e296c73059e1624bef4f4df28401f91c5099f3c80c772dcbb63446e624c045e4ba92c582deafe5832dd6615917c6a9b27a9b6d0d31cbc14c56f55ad0d59cdd7bb051406955318d5515de8a3356b5ec2dcbed197c5967cc04a7016c9e753569e0aad95c3171ea5235bb5fc3379589291b1978d9188ec5e320593cb2403caad98015ad629d843742561647ab7756451b043398f3dc683ce8fd7222d174ac97b01044a3c07cc3ba241d6cbb599bdee1ac4a54ccb09c36124be1436429dc9dad1aa0fb01d1f40d935c1d3626c73948e37db0104b6421e6803c250b997ed7caee3b89c328591cf63197220b71088ee2eb9079ed2286330d0f67b42c3f537f98ffb894b62dc6603295b792cdeb37e42d601cd7b3dcda400ca3765ec355b2bac3a0efb635a1eb3387496c237c407c162b0b07dba8b7556a85959f7db1dc3c1b2b3f15ac7bd14a7e57319f024d3d2779773f599e62370f4fa24bcdce846f790ea0abe5b30430e3d0d49fcb527f408c6696f3a21172636469b51f5f702f46294f66b0cfc593258059bf874aacdf963489700beb77ac4cf868bef43eaecd3e1d17bc3f967dcb64d9378355c6347394b24cd2e228e38ec34cf8f952f7aa03137e3ca6c502ac10d2be7d39cbf60de4561d298ace74c9c37459e8b63b928fe5a372493e9adbd5593e2ac0a27db8db680765b5d298644f741307d89dd54c8b6cdb6715c824b778e16335532196812b65193893e5b35af2b38e51a319d849e62dc2b25895248bddd2cd59162bc6842f9608df94e94cf804cc6e3b02b3db9674ca6e9b88a5c791b2f4d89b558ba5ac52a921a9dcc8f6c334c67a5a497e5d4764b4a1dee3b0a3b49b248b593fbe407578ac6175488cf01c5cf8645921660d5b734acd55eaa35bc70a537e940147c38eacb17163962e756c962e635540910db33411b1c72a8e295876582ecf0e3e27d9612a560a8e969582deac14d086b48ecf6ea4f678438743dd492e3f0073ad5627c0b53a0d13be2201c20fc45ad81ab72dec41586a1c234b8d6cde6f2e24ffdb3b6e77de137430167aad2cf481519e20630761a3c75eca161c45ea48c304731293e918e13ab7840fc174acb7225c9420c22558e82b65a10f92109ec57c7e06db3a361994caa1151ef5473a89cb0c8cf12ab78c67625aaeb6625c9c20c6a558e82159e8b9682eae608be656f276bcb663c0a8de516c6661a6d21a89a9b4a2a73353693626bc41d61f75287c0ec6b111b357ab997f3cccd26d3d6dd01c709a8ba9d22451e55387aaccc384374b8467f67226fc508cd35a19a76e8c53c47274c2e8304c8d16891abb1caa51865507c7baad0ee663805af101063a764b6da45256f8d63bc2b400c31496605adddb19a68598326d32657a3265663125c26c9489960bc36672a252b92cd42cbe35795cdf500f5cf0e1989dd49e003b6991acab313679cbb465533f2ac9930dcc3c87de4527735cd64bba117d2cbb111b62966b7736665817a102cb951b65b9b207f32d34f32d104c17d93a87eeb14a2ceb1c27c93a7e8759a70a53e878acce8828d419bfe562cce97002e674a01bba771fe6d4e9700456379d24a99b76f47356372dc1aaf093dd56e147ca248fe4dbdcc7ad7f8a71c14761824f7529782906fa3409e8ecfece402fc3849f2e33ea1c0a5f8e95ab331250ae8ec6cad599b272d5871bc713c9ff3a5e5175de30afc6c23e2bde848736308c671ae19d8fc70a2c1e675b31303a08d42e31b649eb9cd3a3064bdc4d09686f8fc12ab173b04aecdc4e794e6bb1427b9ecb425b8795abf325e5aa6da0b372558f09bf40227cab43e12b31e19b25c2e90a3b27c257c9ec2836aad79252976d0558dd1c22cd7e64feb3d10fa4532bddcf26b850b64c779cb93947b2e2388e0d8c86d9a95906b9302db059cc3717391ecb80efc367a85dc46cc164c61cb3195763d5e0c556d5e0a4843405212cec4be27b1a23bda91282be8677149b993fcc79cf6a0d56702f95d50ab9ce0a6e0366905de6d6206bc416ee5eee72e16e13d6d3bcc2654fb3196371a55b166b6592c7f0f31be246b90217dc8245f96ab7513e16937c8d5bc9adb2b6627cea961154b2d24ac74357b2fabc29c1cb08aeb53d9ebf25ee3202f771c5eadab02cbdc7f3e32de266d12d7872b76119e97ab719691d56c3dce0b28669c76a98ad2e6b98f5188b1bddb2d880d53037b9ac6136622c6e76c9e2388cc52d6e591c8f49bed5ade41330c9b7b9957c2236b5ee769753eb4ec20ac9369785e464cc40bac38e600b7be7140cf29d6e219f8a49becbade4d330c9dbdd4a3e1d937cb75bc9676092ef712bf94c4cf2bd6e259f8549dee156f2d998e4fbdc4ade841594fb5d169473b0283fe036cae76251dee932cae761517ed06d94cfc7a2fc90cb285f8045f961b751de8c49dee556f28598e4dd6e255f84497ec4ade48b31c98fba957c0966c23ce6d284b91413fcb84bc19761829f7029f8720cf2936e215f81497ecaade42b31c94fbb957c1586f9199798afc6043feb52f0359899ff9c4b33ff5a0cf2f36e216fc158bce092c575188b175db2b81e63f1925b1637602c5e76c9622b16e557dc46f9464cf2ab6e25df24f303e5a576ee85f95764a683d3b917afc9c602d86a9a78b32bec84dd9749aeee38f108f3e1bf6e6b3e879d10317fd1cdb27c91c78f8d170104290049dcc5dcf23a9e5b6ec18625df48c0b0e4ad98f0371320fc36ac04bde5b604dd8ea5c1db6ad2601ba6c93b6e35b903ab18df755931de893512efb96c24eec258bcef96c5762c337e9080cc783726fcc30408bf0763f2915b26f76239fd6335397d07a6c9276e35b90fcbe99fbacce9f76339fd339739fd018cc5e76e59ecc4247fe156f283983bf54b97eed487b0287fe536ca0f635356be763965651716e56fdc46793726f95bb7921fc1aaa9ef12504d3d8a45fb7bb7d17e0c93fc4bb7921fc772c7af5ce68e2730c1bf7629f8494cf06f5c0a7e0af3f4fd563633a3bec30636e79ec09927b2bd3ae6753c195926b988ada6ae27fd8b76f6743e588b636c02bc8acf3159cc669b843a564019cf457a0af048b6e86fe0aa6df1adc5248490c3f5d54f639c7e27e354a598139d0333930415666b909bc03bd684dc737806e3f07b198703d1d1f7d83db70cedccfd805649f6e9322764c64a93edeae544ab6731adfe20d3aab42375a3f7638ade95c6609ecff7cd0af3b520e6fe46f6f510532d7ad761275a3e8769f94799964729ccc3735c9775bcb43b21f23c46e4071991b3925caab15d8ee8f3cee26167bf24b5f5c60b18e93fc9481723250cdb212dbaeea07b36949138d63281892b3f2f623afc59a6c301880e55ccf3d6c8779e36fc6b661db198cffeacb3cce189d3e8254ca3bfc8349a86d6e6b27a2b3a4dda62e65ae1b560ecbc2c273abd8ce9f457994efb396ea18cba2bf65bfa5da5a398be82c5f46fb2981e2cf871a353dd881b5d066550a5e57a232bf7abc04c62eb9c93b87cf52aa6d9df9d590978be32acc5127eec79bba4bdb0d6ad3375d86b9856ff9069b5c0b19560e42c377682933ad389e6af639aff28d37cb690536353cfc8a7f3982ec792976a787db8314ebda7be46fc05a6e93f659a4e72d54ed5b196aa853fb580a7a995dde3449337304dfe856922cf6bf4e816f3a81623fee18e3addb8abbacff126a6c9bf659a4c8c3a94467ec84c6af4780bd3e327991e536de8611ead931a7ddec6f4f93faca5c562135d2e66b19655561b74b62cbc83c5f4ff65312d47cb82ec3c093be78844a74f7dd49e2eeadbad7731edffe35e7b3b6769a456fbf730ede962da18ed37a7cc73534aae667b51c79ead6552ebc8b3f6fd5ef0485eab5e62640333f9f6658920ff3e46de2f237f98d09616778ca8c78eac475b114dac0da5b31542965e222b6bc92a34273a7f80e99c26d379665c9dcd234b0c8d8dd2d5c64b56d8911689b3913ec474ece2477a8d4e742c21b9acce95669dc9ab1f613a7595e934dea14e66cb1659322ac6397214b493587f8cc53a5d16eb12a1058e8409cb53195fed5ac98e8e6bb688297e3f5139ed134cbf6e32fdd6384c954ad647367a5d8ba37622c3f4924b8fddc9c57ede354e98481cb14f3162193262458eeb5c3327c79e8c91289fec67980699320da6dbcad3469d1242fd20cef374e7caebe7986e597ec4e3e3cc8360a64ec4fb09fba02a3d045f609a75976976a463cd2a24be1127bab81971e94c7bf225c6a387df81af387e4acf8bf1ee1ba4da6cd9d74e34fa0ad3a8a74ca3099df28c98e342916f13dd627e8d69d1cb8f8cf4c5af5de6b05ab086f70cbcd06e7e8369d95ba66599c23ecf22eee1ab675ae17b5c24a2cc7d8b69dd47a6f541965adbdb8d23ba2c46ef1692b8d4fc0ed32b1bcbb3f2ba2efad8bc12f2bb959533430bf39b6acbfce94cb25ccb0cf2e367bbd2cd62b2a91556ced2dbd83bbd801d69107470aa178d6f13a9c129dfca0eef42fcbd52fa1280ddd2f8667e63fc5dd9be8e86092739567a2693dfc8758d9c273a87950023bfd492cfa695e9eee49e7952edecefae239e53e0673bda40598474507e80b6de9af6a3a9ba374fd5b4027610c3b84ee6a6f8b1c8a1a17591e7a1d8d324a0e746167743803916e0df0bd9aed810806f591c371ef14eb5c86302ccfe4c5c7155a09a90891bec374e03a366782baf20bfec41cf754d0bd2f4e9af12a27cbb481d1075ed1b41344025a25e31f98c3a1ecd4ea00ea8820323a806aa44952dece86ceec54ecf0459c9fbc43a005b961b01364825b09c8e13a6ca59e0b869a503b64d209fe5aac4d607608bf4870ee57612eddbe9806ba72f826b70726bb036df11ac0bd1463a643aa0fa292b826a2f95a80ccb5bb647b90e987eec15c1344425a601921c651ac6216665b5b20339366a53e5efec1e41375425ba9e00dd2236072dc485e9016a5bef08a861aa8d09d37d50c272d65a521c67b2b6b1911bac5a585fa09adf5b7d352fb68a656c34a24d0b5467a445500d5789caed11763ac05c01dacc112a611ae7b8843bcef9d5020e28942355c2c9e4aebc3ae09bd001d00fe91140fba806241e71a303a0b9c0d61aa512107688b60e902eeb1281345a25241a74f4895b5ab4773d2378f65589c7f9f1793ae0db00bca4fba9ee15ca4f09d201d3557d2298c6a877cbcc63f9a7958930863f8d0ef43c4ddc32d90322b802aa5b3ea3ce3246f675e9d0cc05b5d6589580e8f868c42ad7c26a0268f6573d6e1399e7a253f1fab27f04d1389588060244ad7caa4c1d6beb2a795fae9188a41352eab471bef840b765bc4a787d250683b9be462fc7de12e0d8cb5389ac5f87fba59457eafa8ee4bc044672f25542eb0f7c566be3cc83d2015ce9a008b80295e08ce3e7e9849e868e810a0a8e16d0902686694bbf08aca04a58b9965345e467b46b61aab20e5046ba207c76c2262d39a38e4fafeac3bc601bd8a600ad4c5e9d38dd8aceb7f75ccc0be174a980639322606b102310e3410dd818e10ec4f1f70750c75140ea940cb8768a07e20c34076c5a150187c38b814e95ef40279c22814e4f3a0b487b7b0189df2a20747b02520f69c062a43a6039c618b0f0660490f96081b8cd4d00e991046c38050271a658051c4c9309d836c3028e6c8f80adb94d91d9bbf259b7413a7bb7e87f7c626d900e881653084f99f35099053021191680515735b3bd8facd6561792765888c3d0389b85c3a3cac5d86445d90263ed28d8297f32e81e4ef412ce222d715e0c704ef212ce622d71de05704e4e06ce5528d0aaa8a51b85be49ea7066a9c2b906e09ca27ac6528dbd09381ae4c27e00db54d5fe47ab421dbd5145a13a9419aa509e0b501ea07a2a5864a59c6139ceef585568d86bcaf0e5aac29705f04d53ef64abe69ba68463b60b8b2ccd57980be7aac23806603c3095053a7a9d74501dca0a5528d70294072503259623a3ebc6a08675e3de00e5c1eaa73246e74abb7b872a45bb4515da5680767a6a7329dc504a21ca1c552827019487a85f19461b986a61d78d16e64da18f29c3374015be69005f89fae1d8509c0d2df433c607028033bce5ff99a421ceb301ce99def2ffe888f34a80b3d45bfe1f1d713e0070ce5289735f8eb3913cd4c8aee6f087b94152f4e626e67506f9960edeb5333ffb3af6a4c21e510f55a0ef03a067eb023aa821e83b00e839c998bf6dd7a33955437fe6020073ae97604ed110e66100e63c2fc19ca821ccd900e6a15e8239414398b300ccc3bc0473b28630e70198655e82a9e380da5c0073be9760166a08331fc05ce02598410d610600cc855e8259ac21cc6200b3dc4b308b34845904601eae7a2d58085be8a481ab3817805a948c5c870d5a88bea4a0869ea4710066859760166a08330fc0ac4cc618109de5db1e75a64012c68094e13b10e0ab52bdbd4f0b79604f99fe9209c02d560dae356a8e3d5ccba00cdc3055e07a007047247b6a817d87ee640d1dba5703b44bbc330166a2861360f607288f4c462e8528ed1f46a3d0fcd9a00aed2280f6286f4da4d6b1435807702e55897330f9606fb9113c484119d08b54015d05802e4b4d0365e7643485c6faebaad0de0cd02e576dac1bfd6ddbfbba6ad00bcf00f88e5689af186d94ec9e23133dd10b9e4449fb4ae64a4b85cdd72c5589301a2442b5ca44e8c27a4c4a97a928b344bb00482b54ef24649e8a3c8f99f4e1d8ed0a12076ca42a60bd01b09a54fa84c4753d933434dee70098c77809e6640d611e0a60d67a09e6040d61ce0430ebbc0453c73e7a298059ef2598451ac23c04c05ce92598c51ac22c0130577909a68e6bd10e0230577b09a68e8b9e0f0630435e8239454398f301cc355e82395543980b01cc06953047c471bbc53f705a612d3a5d15de0701dec6641cd7d5ca7672a3704dd92195535fd25581839bd6367977bcb248432fd14680b6d9bb68a76a88f6068076adea5121b888ca74c6ed693b229c0c80b6a4726c5dfff9c04300ca63bdb56c5ac71970a7019cadde5a36ad23ce0b01ceb0b7964deb88f33680b3cd4b38276889f35180739db726cfe8b89caf1ee06c577fb890317bb88ecfe648d2dce16255f06a01bcf52ae105d085faad515362f6ecc933f702dc1bd41f0562e4d5d83db1cdbe7d9b8613686a00c28dded96caf48c3cdf6ae01288f5389924e44af95eee3bf88fc6e03d30e65fbfa2b039ba70a6c7f00f678ef3a4574dc59a61da03d41f59ec321bed025cea9531ad49add00b613bdb3ac40c751f69100e549de727d4cd5b0377401c079b2b75c1f3ae2dc0a709ee22dd7878e3877019ca7aac439dcf626ba73f82b49b33f952d233c09c03dcdcb700b3584bb0ec03dddcb70831ac26d0470cf48d68af53d67d7e20300be33bd65424dd0b0913a13e03ccb5b26948e382f0738cff69609a523ce7b00ce4da9ec7aea7f3a4610a03cc73b2707e9381764134079ae378ebe096a78f4cd7e00e379def1c6176a58b8370094e77bc745a7e37ab74280f2026f8db0ebb893e652807373327a3bc5a9d89f4bd918fb5100df85deeaed4cd6d09e3c07e0bcc85bbd1d1d715e0b705eecadde8e8e3877029c97a8c439c6d1fe1ed40db78e8984fb78506a4930e297a9823d19c0be54f58298d82d2a2a7df37de5e46f65e0f655052e1b80bb4c25b81ee8cc0ffd9cbe3d01b4cb5577198d5dba222edf43c91dc38b5e17755fbf556c2300c62bbc3b3966828693632e0568af548976a404ad31a190c26b26b99536362d7bdcb2973400f82aeff4d175dc46655f80f26a6ff5d175dc067f19c0798d4a9c038489dc210166928e0d513636b91d80bcd62b20831a82bc0580dca27aa626669c2bc3364a15b63e00db75aa4ff40db1139c4c64545c3379859ae92bc98f52fb52590f7c3000787d3226bec03516adcc7e6c661b95d6f1f3ce43e4f50a665fd675ccde50bcfa628c2ab87b01b8377867bc678286e33d7702945b55a2cc899a43249e8f9a14dfbab21e4f398078a3773b93c51aa2bd04a0bdc93b7d1d1df7459a0a50deecad11a0220d5bf8d301ce5bbc3502a423ce8b00ce5bbd3502a423ce6d00e76ddeda2d41479c8f019cb72763797feca667cdfc11fd1a1e3f80b72d352b7c8da34642a959e13b4e15d81c00f60ebdf64d086ab86fc2ed00f79dc970ca6175aaee4eb94700c8bbbc338358c775c0e70394db55efe5210e0797b146aa4df3a1e0b100e1ddaae76fd43263883e3287353d75e4c17ab5cd4f5f55e0ba0270f778c70d57aca11bee3a80f2de646c000dddc561ee71a7cee252b0a59462f7b0b2a9ae4702983b54c21c2d98456bd9a9aa864773cfdf08fa4480f93e959833f82628c6f8108daa7ed3b67a0158f77bc78fa9e3c903c301ca0792ed6d97af293786d828ee954c5c935ab45b54a10d03b43b93d50c55b3b3fc1ad8927cb3fea43b1d85d8d42de5cd90329f511980f9a0b7fced1335c47916c0f990b7fced3ae2bc02e07cd85bfe761d71ee0038772563755a512a56a729c33703e0db9d8c530660ffa7849deb4b8df7990c682331d10d2fa5be8dcf5000f49164cdeb08b113cb6a1987f63d20571600888f7a775ec7240de7755c05d03ea67aed8fdcc5a19f63631080f67832670537ef610310b702904f7805a48ef3d4ef06209ff48e5b43c7b3a6c703944f256b37b7f5ec2cb536d2a424a9adce51856f0ac0f7b4b756f2e8b8c6ec0880f399d4983e7bea34829b00da67554f8711fb3ad4445fc0fe5ecf5e6c2042ebd9349946661ce9dce75902c03e978c3cdbc6bc94e1ff81ad461703b4cf27a3712adca3dc1bd301be17bc73be6f9196865208c07cd14b30755c00bd02c07cc94b30276a08733580f9b29760ea38b3ed6800f3152fc12cd610663580f9aa9760eab86b6b0580f99a9760ea389ba30ac07cddbb3ef6891afad82f03687fa17e0cc8ceaa73fdd69af70510dff06efe9ca261fedc02d0bee9ad0932410d3b99a7029c6f796b828c8e3837039c6f7b6b828c8e386f0438dff1d682541d71ee0638df55bdaf91b85f4719ffac0c5b5015b60100db7bc9de9b50ee245ecc1dc3545431896e2b339c36aa1d10563645e17100f87daf030e6a08f86100f803af032ed610f09300f0875e075ca421e02700e08fbc35523f51c391fa9500e7c7c998cb8479a3749fcb5409407ee215903aceae3b1c80fcd42b208b3404b91c80fc2c198e66bb7da6291af6984e00303ff7124c1d4fa43e05c0fcc24b3075eccb3700985f7a0966a186309b01ccafbc0453c76dcf5a00ccafbd04b3584398c70298df7809a68ee7f9b60198df7a09a68e8b57d70398df7909e6240d611e07607eef25983a1e64773c80f94bd57b7385d958fa4abe29245d2c389f6ddcd3a00edc1055e0ba0370bff2cede5c1335dc9beb7e80f2d7de9d0112d47006481340fb1bef2c1fd47186e73e00e56f93b1aa28b2f865cf3e81e01800f6775e055ba421d80900ecefbd0ab65043b0a300d83f7815ac8ecdd53000f68f2ac166f21d0f1b7c47742cd0d66f49763ac0f583775af7291ab6ee1301ca3f796baaac8e6341e7019c7ff6d654591d715e0f70fec55b536575c4f910c399912e089f1d237c2653b691f7ae43bcf6cbf7cde998a91922dfe477ccd75c418aeb52f2e32c0d96931fd97d3f3bac64966f03891ebddbc6d67d50d94de45b1ab3b02f58441ef92bcd12d93c4b04fc347318079a04e26c891e903684013627d5b83b9f215b4d343477540eb025e4c6b7269f105f7d029feac56251cb92a781397ada3831bae8dc90722c5b7c1ee6277aca9eb2761b19df1bc94a97afaf67b650e4549280c5c9f401c9b6c7017f96f494d180e5e9a3462cb06fabc8f7f6d7ef1821d5201be6d14ceb67477a9a8778ce675b49d0b7cdbd3b6a48dea0fa16d0f965fe49e4976133e677588b46be6e223fc627f37e7c9934975a4bfb1bcd8a7e79edd498b0f2154f23a34aa867b9c6c8316176379ea6f1f5c3cb6a4f8b5429a05db13d5affbfd3744fe3e99e5640cd297f6102f25efc18fd03869cb2cdedbc97b6e5acbe69614fd268bac9dbd4d91d2d2fc4c22d98e4cd7c9d38dd7f84f5595ac1e4cee66be731faa7ca7c6d48339c0a82ec3936b493a783a873e4a916961f6a3af28251ca5793bfeb6c9018c6fa9fb834babdec2aa66b88496df299ff687b49ed665a1dfd4b25cff40e93681e51e1e7ba815aa6b46ef87754f109d2e2f3d3cff93a41f99afe0c624d541579b78edbce0bc9f3862dd99f7c93fe5f842482692de50100
```