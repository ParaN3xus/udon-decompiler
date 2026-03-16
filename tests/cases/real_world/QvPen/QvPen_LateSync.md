<!-- ci: skip-compile -->

```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDK3.Data;
using VRC.SDKBase;
using VRC.Udon.Common;
using Utilities = VRC.SDKBase.Utilities;

#pragma warning disable IDE0090, IDE1006

namespace QvPen.UdonScript
{
    [AddComponentMenu("")]
    [UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
    public class QvPen_LateSync : UdonSharpBehaviour
    {
        public QvPen_Pen pen { get; private set; }

        [SerializeField]
        private Transform inkPoolSynced;
        public Transform InkPoolSynced => inkPoolSynced;

        [SerializeField]
        private Transform inkPoolNotSynced;
        public Transform InkPoolNotSynced => inkPoolNotSynced;

        private LineRenderer[] linesBuffer = { };
        private int inkIndex = -1;

        public void _RegisterPen(QvPen_Pen pen)
        {
            this.pen = pen;
        }

        public override void OnPlayerJoined(VRCPlayerApi player)
        {
            if (VRCPlayerApi.GetPlayerCount() > 1 && Networking.IsOwner(gameObject))
                StartSync();
        }

        public override void OnOwnershipTransferred(VRCPlayerApi player)
        {
            if (VRCPlayerApi.GetPlayerCount() > 1 && Networking.IsOwner(gameObject))
                SendCustomEventDelayedSeconds(nameof(StartSync), 1.84f * (1f + Random.value));
        }

        #region Footer

        // Footer element
        private const int FOOTER_ELEMENT_DATA_INFO = QvPen_Pen.FOOTER_ELEMENT_DATA_INFO;
        private const int FOOTER_ELEMENT_PEN_ID = QvPen_Pen.FOOTER_ELEMENT_PEN_ID;

        #endregion

        private bool forceStart = false;

        public void StartSync()
        {
            forceStart = true;
            retryCount = 0;

            SendBeginSignal();
        }

        [UdonSynced]
        private Vector3[] _syncedData;
        private Vector3[] syncedData
        {
            get => _syncedData;
            set
            {
                if (forceStart)
                {
                    _syncedData = new Vector3[] { pen.penIdVector, beginSignal };

                    if (Networking.IsOwner(gameObject))
                        _RequestSendPackage();
                }
                else
                {
                    _syncedData = value;

                    if (Networking.IsOwner(gameObject))
                        _RequestSendPackage();
                    else
                        UnpackData(_syncedData);
                }
            }
        }

        private bool _isNetworkSettled = false;
        private bool isNetworkSettled
            => _isNetworkSettled || (_isNetworkSettled = Networking.IsNetworkSettled);

        private bool isInUseSyncBuffer = false;
        public void _RequestSendPackage()
        {
            if (VRCPlayerApi.GetPlayerCount() > 1 && Networking.IsOwner(gameObject))
            {
                if (!isNetworkSettled)
                {
                    SendCustomEventDelayedSeconds(nameof(_RequestSendPackage), 1.84f);
                    return;
                }

                isInUseSyncBuffer = true;
                RequestSerialization();
            }
        }

        private void SendData(Vector3[] data)
        {
            if (!isInUseSyncBuffer)
                syncedData = data;
        }

        public override void OnPreSerialization()
            => _syncedData = syncedData;

        public override void OnDeserialization()
            => syncedData = _syncedData;

        private const int maxRetryCount = 3;
        private int retryCount = 0;
        private LineRenderer nextInk;
        public override void OnPostSerialization(SerializationResult result)
        {
            isInUseSyncBuffer = false;

            if (!result.success)
            {
                if (retryCount++ < maxRetryCount)
                    SendCustomEventDelayedSeconds(nameof(_RequestSendPackage), 1.84f);
            }
            else
            {
                retryCount = 0;

                var signal = GetCalibrationSignal(syncedData);
                if (signal == errorSignal)
                {
                    return;
                }
                else if (signal == beginSignal)
                {
                    forceStart = false;

                    linesBuffer = inkPoolSynced.GetComponentsInChildren<LineRenderer>();

                    inkIndex = -1;
                    nextInk = null;
                }
                else if (signal == endSignal)
                {
                    linesBuffer = new LineRenderer[] { };

                    syncedData = new Vector3[] { };
                    isInUseSyncBuffer = false;

                    return;
                }

                var ink = nextInk;

                if (!Utilities.IsValid(ink))
                    ink = GetNextInk();

                if (Utilities.IsValid(ink))
                {
                    var totalLength = 0;
                    var dataList = new DataList();
                    var lengthList = new DataList();

                    while (Utilities.IsValid(ink))
                    {
                        if (!QvPenUtilities.TryGetIdFromInk(ink.gameObject, out var _discard, out var inkIdVector, out var ownerIdVector))
                        {
                            ink = GetNextInk();
                            continue;
                        }

                        var data = pen._PackData(ink, QvPen_Pen_Mode.Draw, inkIdVector, ownerIdVector);
                        var length = data.Length;

                        dataList.Add(new DataToken(data));
                        lengthList.Add(length);
                        totalLength += length;

                        ink = GetNextInk();

                        if (!Utilities.IsValid(ink))
                        {
                            nextInk = null;
                            break;
                        }

                        if (totalLength + ink.positionCount > 80)
                        {
                            nextInk = ink;
                            break;
                        }
                    }

                    var lengthVectors = new Vector3[(lengthList.Count + 2) / 3];
                    for (int i = 0, n = lengthList.Count; i < n; i++)
                    {
                        if (!lengthList.TryGetValue(i, TokenType.Int, out var lengthToken))
                            continue;

                        lengthVectors[i / 3][i % 3] = lengthToken.Int;
                    }

                    var joinedData = new Vector3[2 + lengthVectors.Length + totalLength];
                    var index = 0;

                    joinedData[0] = pen.penIdVector;
                    index += 1;

                    joinedData[1] = new Vector3(lengthList.Count, joinedData.Length, 0f);
                    index += 1;

                    System.Array.Copy(lengthVectors, 0, joinedData, index, lengthVectors.Length);
                    index += lengthVectors.Length;

                    for (int i = 0, n = dataList.Count; i < n; i++)
                    {
                        if (!dataList.TryGetValue(i, TokenType.Reference, out var dataToken))
                            continue;

                        var data = (Vector3[])dataToken.Reference;
                        System.Array.Copy(data, 0, joinedData, index, data.Length);
                        index += data.Length;
                    }

                    dataList.Clear();
                    lengthList.Clear();

                    SendData(joinedData);
                }
                else
                {
                    SendEndSignal();
                }
            }
        }

        private readonly Vector3 beginSignal = new Vector3(2.7182818e8f, 1f, 6.2831853e4f);
        private readonly Vector3 endSignal = new Vector3(2.7182818e8f, 0f, 6.2831853e4f);
        private readonly Vector3 errorSignal = new Vector3(2.7182818e8f, -1f, 6.2831853e4f);

        private void UnpackData(Vector3[] data)
        {
            if (_syncedData == null || _syncedData.Length < 2)
                return;

            var penIdVector = GetPenIdVector(data);

            if (Utilities.IsValid(pen) && pen._CheckId(penIdVector))
            {
                var currentSyncState = pen.currentSyncState;

                if (currentSyncState == QvPen_Pen_SyncState.Finished)
                    return;

                var signal = GetCalibrationSignal(data);
                if (signal == beginSignal)
                {
                    if (currentSyncState == QvPen_Pen_SyncState.Idle)
                        pen.currentSyncState = QvPen_Pen_SyncState.Started;
                }
                else if (signal == endSignal)
                {
                    if (currentSyncState == QvPen_Pen_SyncState.Started)
                        pen.currentSyncState = QvPen_Pen_SyncState.Finished;
                }
                else if (data.Length > 2)
                {
                    var index = 1;

                    var length = (int)data[index].x;
                    var check = (int)data[index].y;
                    index += 1;

                    if (check != data.Length)
                        return;

                    var lengthVectors = new Vector3[(length + 2) / 3];

                    System.Array.Copy(data, index, lengthVectors, 0, lengthVectors.Length);
                    index += lengthVectors.Length;

                    for (var i = 0; i < length; i++)
                    {
                        var dataLength = (int)lengthVectors[i / 3][i % 3];
                        var stroke = new Vector3[dataLength];

                        System.Array.Copy(data, index, stroke, 0, dataLength);
                        index += dataLength;

                        pen._UnpackData(stroke, QvPen_Pen_Mode.Any);
                    }
                }
            }
        }

        private void SendBeginSignal()
            => SendData(new Vector3[] { pen.penIdVector, beginSignal });

        private void SendEndSignal()
            => SendData(new Vector3[] { pen.penIdVector, endSignal });

        private Vector3 GetCalibrationSignal(Vector3[] data)
            => data != null && data.Length > 1 ? data[1] : errorSignal;

        private Vector3 GetData(Vector3[] data, int index)
            => data != null && data.Length > index ? data[data.Length - 1 - index] : errorSignal;

        private Vector3 GetPenIdVector(Vector3[] data)
            => data != null && data.Length > FOOTER_ELEMENT_PEN_ID ? GetData(data, FOOTER_ELEMENT_PEN_ID) : errorSignal;

        private LineRenderer GetNextInk()
        {
            inkIndex = Mathf.Max(-1, inkIndex);

            while (++inkIndex < linesBuffer.Length)
            {
                var ink = linesBuffer[inkIndex];
                if (Utilities.IsValid(ink))
                    return ink;
            }

            return null;
        }

        #region Log

        private const string appName = nameof(QvPen_LateSync);

        private void Log(object o) => Debug.Log($"{logPrefix}{o}", this);
        private void Warning(object o) => Debug.LogWarning($"{logPrefix}{o}", this);
        private void Error(object o) => Debug.LogError($"{logPrefix}{o}", this);

        private readonly Color logColor = new Color(0xf2, 0x7d, 0x4a, 0xff) / 0xff;
        private string ColorBeginTag(Color c) => $"<color=\"#{ToHtmlStringRGB(c)}\">";
        private const string ColorEndTag = "</color>";

        private string _logPrefix;
        private string logPrefix
            => !string.IsNullOrEmpty(_logPrefix)
                ? _logPrefix : (_logPrefix = $"[{ColorBeginTag(logColor)}{nameof(QvPen)}.{nameof(QvPen.Udon)}.{appName}{ColorEndTag}] ");

        private static string ToHtmlStringRGB(Color c)
        {
            c *= 0xff;
            return $"{Mathf.RoundToInt(c.r):x2}{Mathf.RoundToInt(c.g):x2}{Mathf.RoundToInt(c.b):x2}";
        }

        #endregion
    }
}
```

```hex
1f8b08000000000002ffed5d077c14b7d2bf3b30d8f45e4233dd34770ca440a8c654879602c418db1883b11ddbb4f4de7b48ef09e9bd93de7b4f48ef3d2fe5a5f797974fd26a7d73b2e66ed7bb5a4bbc0fffb0bc6d34f3d7481a8d4652242344fe8547935f4b420b425343e9a1c5a1d25075a88afc3595a4ebc88f7565df2f0cd592b49cfc2e26cf468752137c49ff0d0ff722bf0bc8555da89e7cb93e5442d20af67c61a88cfc5d40be2d2357f4ee2af2bf8cbc156e493e5a1c9a169a1f9a17ea19ee9e80c412f6511dbf17a6728593c9ef29a1cde46919638a6692c11ef524bf16922775ecd93ac2b4fdded2d07226d63af2ac847c531baa243457328ac9df8f0cb13f4256d28da72d78da95fc4fe17f47f8dfc9fc6fec9e4da33ba095e2e0fd963cede1f0fd249ef674f87e6f9ef602fca5703a731c7c7f0ef9df8afc39065cd3b40f4fcfe6cfd3c13591295ccfaffb0aef6508ef6de1d7fd2c9e22d73be0e93c81a7f3049ece15783ad7ca2b922cf074aec093fddea9fcfa7cfe3c935f0f02f7697a017f9ec5af0782fb34bd903fcfe6d703789a0a9ed374307f2fc781ec43789a0cca7f284fdb817bc32c3cfbddec80a6fdcd70873a956ce1d47219bf1e01e8503972f9f54881fe4540f792415d83e9c5e0fd519cde58e19dd13cbd943fcf03d734bd44c813627409ff669cc09bad3b1d84e7b6ae5c26e8ca651c8345fc7a8c2557528d95b6dcceefa7817c521cd2b4cb2c1da19921f09e19c5b57bae83f26bc3d3cb018dcbadbc93faf23c7707ef521ec727f896a6590ef5e70aa1fe5e2194c156a1fe6ee5bc3d25e0b755c08fbfd76a05bfcee6b87d0ff86bc93e61f75bedcaefdbf52e57a8c74eeaa55d1fdb02d9c7f2ef2638c0a22de7690be7691bbf6feb5f1ed0235ec62d321dd01dcfdf0d81ba2dea61bcef27083ae636ff5d24b8ecccd32b393ef09a60d0edc438ed995d569310b92646f91bf010bfb71b4faf0274aee269279e5ecd79d9055c135e92c7233212fa29a70ab43af0f41a81d6359cd6a70226b0edb6fbff1bf9b7bb826b9adaf5b0157f6ecb3499a7ad01ad293c6d1fc5a211af1d797aadc0ebb5162f0def0f15f29dc8afa70acf6f12dafc9be2e88b4c27643ad49ea7d781f7aee3e9f53cbf49e09af2bd9dcbfb3bbf3fcdba1e743bbf9e1e87e60d02cd1bb83ed60bb2de0cbebd857f6397cfadc2b54dfb3681f66d16edf6369f3384f79fe4ef4f06d7349d09f2b6dbc10270ef699ece02f79ee1e9ec68590cdf1d7c6bd34a01dfdab452c0b7362dfb5ebe2543db9916cdb6364e7312604e9eb5b1fbcbb9881d301f7c437198223c2fe4e93ce4f91e4006d9f305401ed9f38588cd61f3fda060733c08f43e45a803cf827bcff2f439fefd54e1fef3fcfe34a05ff0fe74f03d4d5fe0f76700fd83f7a7039d85dfddcc9fe7f3ebc52eeac98b822ebfc875b9d0a2d1be126f871af4dda67507a73553e0f30e9ede29f079274f97f0f425c1467889f3f2bd908fc803d4c15b055e0a041e4624e0654f9edec59fcf02d734bd5d681bed76e46580adcdc32b020f2f83fb10fbd9b1d8771a2dd0b1bfdb8ba7af0a78cd89c5ab633b0b938eddacb4534b818e2de3dd828cb783fb347d803f9f8b7c7f0f7f3e4fe0eb5e7e7f3eb8a6e9c3fc7e21a06fd381cff740f27b08e1f72181df71c2f77dc0b55de604970e5b2472c37a3c42c0e31e4167ee11f4fc5ee1b92df7364467ee033ab3778231d4fd2ec650587bf600a0b14de045c4ee3e01bbfb04d96e15caa440280bb1acedfc1e11307e84a78f0aef3f0ceed3741f9e3ec6df5b20d0ed233c772a875df68f0b7cdd2ee0b34da067bfbf5078ff0981ce7de0be2c7f59fb718bb7f6a35b9a40c7fe6e294fb7c76f3fbaf23ebfeb222bed1612be7b8d7fb7085c637de3eb48df88e1faba80abfdfe1b08ae6f20b826a8fb5db60b36df62c1e6b3af97097ce645fb9e5607727c8ee4cf965bd7fdb7f1717b2f30e6b1fba014a13fea03de81fcdf21b47d570b65cdc7340d79eceb61bc2bfa8cecbefe4dfeed12704df3ace4f26d15be7b4b28a3b78476f44d41069bde5371782b02fe109a1647f11f689789ed1b781bf0b212694fdf11ec9d772c1e7ad83c9420df95823c64366619d226afe2e97b429bfc1e4fdf053cbf0b784a89e5af9fdd57ad46f82be7e907423e1ff0f4437e7f4f704dd3f7415eef0be5f511ff662f704d78e9b9294e795508e5156f0cff31c8fb6361dcfd893096fdc4cabbd74c8157bb1df954e0f553febe3d365a9b00b735a05c9300de1f0b63ec4f05beec7c3e14f8b2ebf5e7025f9ff3f7dfe4d7eb12f05589f065e3fb9950e73e13caf073c1a6fe5cd0279bcf2f04dfa37d8fa65f0976e0573cfd17bfbf37b8a6e9d7fcfe3ee09aa6dff0fb4bc1354dbfe4f79721f97f27e4ff1d4fffcdef2f07d734fd5ec8df1e3ffc20e4ff034fbf15f2ff42820d6c9b6dfe7e14b0ff16dca7e94ffcf9bee09a60bfd3b638f5e74ba1fc7e11f2fe45e8177e156ce25f79fab36073feccd3df049e45ac7f16ead56f427f6c3fff5da0f305b82fc3cca6f707d0b33f0499ff14fa873f055dfd4390fd3f82ec3f83fb34fd5bd01bf1fbff0ae397bfc17d9afec39f1781eb309f6504fa15e6f651381cab5fec9afeff4bd0afbff8f38850464259842302767f0965f1452cbd469857216d4b8d90bfd897edc7d36ae479add0d7fd21d4973f626dad3e5b1dd81e75b17a1f6e2160531f7f7cc466a41d8e8fc2ad62fb48764dd3a4288d708b58ecd93330ae68788ecc31353c17edc54438ac1770682de0b021010ec92e706823e0c0e781c2603e81e50f714811e46c2de0d01179ee168789b1b668b82da7bb025c93324f4d8b7d3fdc2eb65d0ab78be5af814eba406785906f1f60abd86d08e13db55e986bd8cde1dccb26419ef6823ced391fbfc7be1fee20c863db479b053ae9b174061e2bd0e928d0e928e0d2893f2f06d7309fceb1cf1be876e6d71b63711a788180d3468738150b38751170ea62c9372835f6fd705741beae827c5d049c6c3af6d86d7f21ff4d9279e4cd511d4e3d15c8658f41f8b341e708b2af70283b9cffb153caf34ae17e1fe1793e98d7a1b27513fae6d6e03e4dbbc7f6b3ec9a6031f8f6583a0ddff5e0ef9780eb308fd4016329764de95c1ffb1ef33fa7e0736f83b60be3cee91e62750eb2680eb5ed6bbbad3998a70770de7a71deed71dd21fc7e6fd0f6f502f7e8bb650ef23f2c41fe87729a3b09f91fceeff701f9ef04eed1775739c8ffc804f91fc169f615f23f8adfef07f2ef0beed177cb1de47f1c4f8fe1e909d17a31c2b6f3ecb6e9789e1ecde9af7640bf0be7a93fffa6025cd37400bfbf065c139d1c961bfb3dc3c39e3fb3fda627f1b4339081bf33d46ebfc5f96c9bf7b5c273bb8e9e8c3cb7edfb5390e7b68d7a2af2fc349e9e2e3c3f4378cf6ef72a11f931bcf9383b9c2ad4f1540bcfe13c7e65b85d77b748e62fb748e62fb708f397700e39df214f03394febc0354d07f1fb55e01ada528385766fb0254bda39cd20c3504186a1427f35843faf06d7341d2694c7304b86113cfe65446970b2d83cb13c284f35c2fd59fcfe7ec2fdd9fc7ead106f93df8476ff049e9e09ae29edbad8e7e1e1fc7e3db8a6691abfbf5e787f04bfbf015cd37424f2be6d6b6f04d7611ec00bdf3f0be46bd3b3dfb3db44fafe26a7382485d9d86766a88cd82e35a10cf6d2081761c3f697890386d9783162c5d88686b36ceb895154430848a27617b398e0fa504e285b1ab5db9bd9e676ee53d9ef62f2460523ba39dc288bbb1b654199ab243f6520e4b88edccf6711cb34e8b8823c490fcd2129fd6a45282bb494fcc4525942b2ad2403afb2d022f2bb86d15b41d86efc6622916878b2f8cd0242d38a9e5e47eed880d690eb4ac6e142f67b03e3b48c715f40eed5b358eeaa5039318aaa89314a736b2c7b2df95b96e7220e999cc7e5aeee5ba1da76314d23d250ee33d8f8f6ff4ba4394a84b94636f358f796196c76fff84625f1ff185a8d4c12eba02c1a59d6f280987bd9a10c167fbfa01182ee656c2ca15cbf2dbcf342b971659123438bbb0f93205ac29b6bf69e31323b797352ac6839a10c3694eb9b40b478c0527a696c24e74684a4a49699945111feb00c7e36cc5c1820fcd67b15ec3df705d08a757eb105c086197b10660a593b17ed401712c234ab1a42261dbc51445ac062469a665245de6a547699adf8103c4dd2b92612012b8088ac0092832e80c42d4ebc02486e902b5a00ccfc95229882209888070cc116320499d5ba4c29828d4d33fbaf29e4dbd504890dacbf5d4fbe979972f131a5aedf3102a6723cdb323c53a506a63b8e30845bca1066bee90aa5082f26f72ce3733a7b5ac12c16daefd01575d48e59c5d0b2d6d6616fdb88ce658bdad6333ee263dfce31f6ed19f6635888a72a5eb1524992944a661b3ef3d188fd14817dca3942b895acb8a996b1717cf0c53d87ffb58035d4a58c22fd6faf3ef4afe03b36481e452e834d5864c729623ff9e35e7feb5feb24b9d675665a17144b98feb596a949fbe6b1dc9ad66f51d75b4fa1b8ff21ffe4a8778d6b78b9edb59265e8b1b9a44501a2479f55332ab48d728f2075028f127b7e397cdd197cbd246b8913b18041982283b063f3754b4bf8209b729ea3a075eae1b85bea99b05bf2ce2b562a6d64dd5207be38bab1ae88fd5277bc5f6a8b51eeed95723b1965da94ed24692198652b12ef8a136f2f5353361359a95577eaafb2f671acac7d99b26636a137f34b653bc84a88cd39956bd090f85b2e74eab08568e2b039a8518e9b0bb71ca50ebfe8b0b9648072e8a4d4b7abd7e723564d7fa607eab8c04abfa3acf277e63b0988506552a86c79c85f963c626bd01f6f0d3a61b9a526caedd007dde7d659a6d92c12a3ac5934dbf24657fbdeda0c946975377be2c57f6e527ffab17652ea5f7ffe3931f5cc2d5b2632dd46d47a10536b356c602add45a664adf9e2838443c3d6b83e7595114ee133860909b7c5097793114ee6d3fabd6536ba483c0527de1de37a9847ae7b6043f0e11e87e03d31c2691e09f7c2a018e1118ade58018e943907c312132a4e01ee246bc658a9ae08c4c5b730342d349b3c2b26ef5b1d11bd5f48281693bc29bdc92c87a8532df68bf88dd728c7a6d268d6960c01ae3eaf9c612d481fcc221e23b188c32e2de2beb2e26475e9b0803db6f209f87cb65f5759683e197caee136879d6f356b8e2947f60c2cb54a57719bb48a719178e23ebe42a4cbdc50ac4998ee228c40bd14744b980853cc4ca698e9713a391937eebb3b79b5c862b9eb850d56b1fa61ed64b6a49ddce2b29dec2f23de9d6fbe93b0751f8d131e2023dcc35edd285aad3df9f63c6d587fd1289f2c3c9f54cc70c993cdf1a5b0d6a29edca825bf1be6ed5cd83303652d51afe0a73fad79c3ca0473e9f2d68206f60f10a01970d4d793e475653c3a7f199f074c9f07c94aac17dfdb684023d7093193c5e2198f17cf6059f1f4563f1391a819a14dd57a0611fd7b036f2e16b1bca353c05eba00ba6b533f89e3099f56dda561522008ee31751882f9e87695f8e8c26e7c7443318b64371f7c74c3b05679a2a455fe34e2ae551e2e23de916fb395b055ee89134ec308efee91f0080cebc912ac232eb11e89613d4582f5812ddc613d0aeb40a6ca3a908e6c5d2fdd11b588c5db54b1cd55a19bc94d67321acb7b9a2cef5e6c2d7d11319d8a12f040dfaa656fb8e1660c06f37409cc8b5aba83391d233e4342fc6a97c43330e2f912e2db5d12cfc4eacc4c8f75260b6bf90abcce4e646378cc92e0f165923b3c7230cb6e3666d9cd699265978b558eb9b2cad195550e6a77efc76ed431fbcef2fe17b2a0e4b5e477b94befd5584cd67998acf39b246b1ea663851e756c1ca6097b4834615b2b779a301e23be40d607ba243e0123be50427c7c6b77c477969989cc6754d3ac66e2421e5f5dcc6ce8fd597ca11581bd80d1a6a3da7acfa6e22299ef9bf96f721d18837ef1388a2d2cae639b8d5bc8d551532185edfc12dd55dc029f996772337631336383e41c336577c11af4255e1bf45db1cab0a7a432bce9b232ec864d30ed854e3085627eb99a609a8835757b7b6cea266118ed23c1e83e9718ed2e6b309857728f0087fd4ec2e7e5559eee67912260d0171d192e6355aa7b23c9e2469923c84dc64606cbb1582a17238329b262e917b43746f450ba2b9a7d1d3bf38b104f4ca2fcb1a2998ad59815921ab33ac55d8d9986112f9619072e894f9711efc4f70c4bd888f4c509cfc0b82e9170fd731b775ce763fd43a9d7fe61a6ac1e30dfee9a6699b5f7c3771fbfd6943929e84c1cb102acd35ba5a2d39b85e556ae22b7d9586eab55e43607ab3215922ab3575b7755662e467c8d84f8912e89cfc3baa6b53e38ade663c3d74ad9f0b527f0ed5422c16b45ccdf53cc0f0772338c2dc4785987f9b8685ed9e4ff3a7eae4fd3f3de03cbbb4a96770f9e7716f94f51582bf56d358d93051827d5f1bc6d9493ead046bef4d53f6e1662dcd4c8b8690ff4a3a8c1a1318d0d618a5de5bb08cb77bf78e561e58be5df347fe362ac6ed74aea76bbf6eeeaf612ac6ed749ea76a1cbbabd2746bc5e42bc854be27bc9faf201c1ad4eb3c23d72c85f76f152c3bf9a14376d8ee010008687e424e8afd74ba6a75ac59d9edad068955a5339c3ecdfbd3105dc2851c0873ab953c07db069ce4dd2694e09f138d39c4b65c407f0fdad44983bda308b196cc033588641b3bf049aca6eeea0598e113f40d6a9bb24be2fe6b23d1073d91ed424976d1126c4c11221bee9ee4e8815980577880a0bae1873c91cead125b312c3e83009469d7ab8c3a804237eb884f84c97c44bb15ef2087cf2c1ea1d2b7cb310ca301e8e94f1d02ea6a79e4a6cfd32d647538bc54daeabb05c8f92e5da5d6a1fc4e6de34eba01ce3e368191f9dd8109d2e53aee5d10976708f15ee53ef72e26735a65cc7c866d05c2a570546fc5809f10b7aba23be06b34d8ef3210e732dc6f9f112ce537bb9e3bc12e3fc041f8663eb30ce4f944d84b9e4bc0a237e928478b77eee885763b09cec4381d660b5ec947823c3b1e47f29b0fd9bd6beed87e57d6aa296c60ae0b36220e67a1ea1d6627c9c160f03aba5a30ead1a0fa3b13a4c734e973505fddd694e3d46fc0c09f10f5daae57a8cf81609f1ad03dc11df80113f53d6fcba846523665c9da5c2b8da841957677b34ae3663edc2393e84b7ed8f15c0b9b2a98141ee0ae00019f17efcc4e0849014e1840fc46af2f9b29adc8a8d383289fd7f00f9f62057f5f6200c9f0b24f8240f7187cfc19818177a5c8c75083636ba081b1b5ddca4b1d1a158015fe2b1800fc360bf5402fbb12e613f1c43e7320c9dcb9b84ce11183a5b3da2732486ce1532a51cea0e9da33074aec4d0b9aa49e81c8d29ffd51e95ff1819e1aefc3cd846cd7eb7c6cd3ed2f60fc2b33c1693e55a597b64053d9734aceddc8d901ecc5ba841a189ae843d0e5385eb6476854b55381e13eb7a8f457402564437282ba21365de56b6a864713345102c4db097a4dcbf7aa36c05185bacd23b6ebc00961b3b4038897b0ed3e93f6c6fa19b999fb669b9609ed99330f5bd4536a738cc9dfa9e8ca9efadb25a9914b32da01b6d3e05cbe736593e2d59281bdf56cd452ea762b9dc8eb53171b7307491f36998017a87c4006de9d2003d1d13eb4e9958c9ace9cc008da7bbe6f20c2cb7bb64b97563073142d3319da4d93ccd617773c9efe544e3dd70b1051b986c53313039139ba0b85b3a4171c854371314676180de2303b40bb7c477260d64360735fa7776c3df6ec03c1b0bb6b9d76bb0cd3998dedfe7c3c0eb5c8cedfbbdb27d1ec6f6033eb07d3ea64c0ffa30db750146fc211f885f88117fd807e2176183ff473c0efe2fc6766578d4e3ae0c97606dd0632adaa04bb1a1d0e31e87429761d5e809afd5e8728cf2935e296fc5283fe595f21558d57fda87aa7f25c6f6335ed9be0ad3c56755e8e2d5981ccf7995e31a8cf2f35e295f8b517ec12be5ebb008db173d46d85e8fb1fc9257966fc028bfec95f28dd8e63aaf78dc5ce726ac7f78d563ff703356e3b7fb50e36f918d99d9b2fea5cd16a164ff659f4ae1363ee935c751f8afb3f1ee8004714989f9c1c6beb7ca8a6e103f273da14ebc8e17db6d58fd78d36bfdb81d53e3b73caaf11d981abfed831adf89117fc707e27761c4dff581f8360ceff73ce27d37c6f5fb3e707d0f46fc031f88df8b11ffd007e2f761c43ff281f8fd9829ffb14753fe01cc7cfa4485f9f42066ca7fead1947f0803ff331fc07f181bfd7deec3e8ef118cf32f7ce0fc518cf32f7de0fc314c75fea542751ec770faca079c9ec0887fed03f12765c4a9a4df785c9ef4145600dfaa2880a7b1dcfead22b767b0dcbe5391dbb3585ff9bdc7bef2394cb17ef041b19e9799d8839b771180fd5734e4dead91fda36ce301b60154aa43733a5ede9d19e5a286d306b10d037e72b4b0c0498e9809ff824c3568e9fd2cd3700a808477515b7ec2b5e545ccb2ffc5ab65ff1246f957af945fc6aacf6f3e549f5730e2bffb40fc55ac74ff5053badbb1fcfe5493df6b9825f797474bee75ac54fee343a9bc8111ffdb07e26f62d5e0bf5eabc15b18dbfff8c0f6db584f4be3847def69dfc1400a873d82f42e4639e295f27b98aeb7087bd3f5f7b1726d19f65eae1f605c2779e4fa438ceb563e70fd11568aadbd96e2c7989e27abd0f34f303952bccaf12946b98d57ca9f6105dbd68782fd1c63bb9d57b6bfc0d86eef03db5f62c43bf840fc5fd8f0bc63d8fbf0fc2b4cdf3ba9d0f7af656391a1416fb2338ded05b6b2895b1e7726c00c74b53b6e97b07cb39d447c6063826f64654651ec1a8f35b19828574806df62eadccd0775fe37a6cedd7d50e7ef3075eea1429dbfc70aa2a74f05f10396412f9f32f8112be9de3e94f44f5863be93d7c6fc67cc13d327eccd13f30b86475f1ff0f81523decf07e2bf61c4fbfb40fc778cf8001f88ff81114ff581f89f980e0ef4aa837f616c0ff281edff60c407fb40fc6fac911ca2a291fc2f26ca501f44f907eb4c86f9d099d0a1adb4f51dee53eb1bc63248f329834818692747786c275b6084477a24dc328c4c618e0a7b9bc24c0a234a3f5a85d2b70a23c3ea311e87d5ad31e0d33d029f8c019fe111f8140cf84c15c0b7c180cff2087cdb30d291647bed48da85910632c78706b23dc676ae57b63b606c8ff581ed8e18f13c1f8877c2888ff38178678cf8781f8877c14a7382d7d2ec8ab1bdb30f6c77c388efe203f1ee1826bb7ac5a44718095cdd2dec2d70b527c6f244af2cf70a238b652685bdada7ec2d219cc14eae59d52cdbc2aadb0e76f7b0d3f0cdc9ccaf3332cec173de0e9c43cea3c48a788ac722ee23234c8b78aa13c293712bb42fc6f1348f1cf7c3389eee91e3fe58059de1b5820ec028e77ba59c8a519ee995f2c03012b35e10f616b33e08eb1766f9d02f0cc6f098ed158f21181e733ce23114233cd723e1611816f3bc62313c8c0cc0e7fb30004fc3f4a3d007fd188171be870f9c8fc4385fe003e7a330ce17fac0f9688cf3453e703e06eb091687256b79a307b1c49e1b8c9d339ccef6d1ca677b79c1a7d1b35fe873d8eb17b0bb39840b37dd4e3a26c5129914d304fb289aa7c56f35e19d725dcb0e7cad671c2f0aad267f57c5e117bf1ffb845a53d5cc9e2876b91b400626e59e9894b2b29ac74a6323b36fd6f2131e2db90bc8f3f90dbb61db3ba035b68e640700f827652626e55e322947a1b6de02967329b3742df9ec73f536347c2e720d8f3d75c37316c6f3de329ea70afa0773851a3899705fca24b30e328ac76dbc27fec8988dc9b88f4cc6028732cee507335590ab4a36fe28d644de1c4cdea532794f6ea86df63e1ff05029ab35a0b56a15db4f908eb10a1ade8c9ebaba80d5a632c2f106f6763a97c63a2b602ac9aa8eed3cbb0e7c338d6ed2cc5bd652f66e09a35a4ade6d8c453d3b248be2110f415c92e869b1f2b362c51c9790af2a5cee529b8b21bf0c6be7e42d00dca9970c2ed8feb114270b55b8872fde72bba3eceaf8414ccae5322917379b7ec5d721efa59d87e1b0af0c877d14e260db29d4ef51cef71a5dc2761da5c7caad445a171c0dd83fba41641c864811d6d23645ff17f2de10d77e2754fdd080f198bc2b64f2eeecdaaeb1fb7d6adfc4be656150cf4ab6d4471b66022651b14ca2390a75baf1a1a9f24312fd28c79d31a957caa4de2daed4ce8e738c2ddfd8e326fd2bcd5d30b94a64722d71581fa115349d9591252bfdaac871ed6b4a1d6d3a12bb624894623555ce9fed7fae6657558d4ab29eb5545584e3554c8675711059d4e85d3712ed864954269328cfb544569fd2f8691dabd7742f7dea832fe53bdbd32f1635d19e988849b24a26c93c549239c809557eda4f89f37023f9244cf27237fdc762d6c254f2f196350b628f8a97f0ba591ab78ff7af9eed8e49b45a26519620514ec3fa3671a38ac62557d460ed27fad60dff9331fe2b64fcefdae476a21cf14438f75bb8916a0a26d51a9954658a2d766f96aa531d6e8a1d3015c369ad0ca7a182a7406c67ecb29ec31029277faff6c96b380de3b352c6e7a484b52cba9a347e0b29e2ed94aa1bd9a663b2ad93c9b68bcb16c4f24cb9e1dd0fbd9a81c954259369baebf2b26cb102d6e258bea88a0423243565978fc9598de965626f76225fa27b3f76d3eadc4c4cb61a996ce35d5b29b16d06ed39ea80d47ecf3b1460d2ec279366b4cb5a664be137d7b330ae6b9bae5fd3580f55c1d06e4efd9a8dc95627936db223d9e6f011a50e73407330f9ea65f2ad71a971742cb5998f5996a0b324502e39f5c607373a6f29698d5ee523627331c4d6cb109bd8446f5a7e426f9ab3b1ba1bc9e661926d9049b6bb235d5fc09e1533de4a1be6019ba326cfc7a4db88cd03baebeda35e41ff5ad6428ce74d329e73841299ca5a4ecbce8fd6c8eab8b35b38ea4d9df7da039361333697e9dcdb55d780b945d9a96ec543a02996e4024cc6fdb1b917e7326256bfd3d9cb7872fbdd822cc470384086c3d20423b5a9ac0dd9dc48f2e87bce4adbedfb781d688a6e2cc230395086497693db1dab9f2be39ebf12896637d54fb11893e0209904c35dda0853792f5ceb0bda4b305e0f7612b903114ae44fc747e0eafd797b62521e82e994937e01d7fb785234ad67db0b93e0d0a6dbd5894a2948bb7a6f4cbec364f2a5b9ea11ec1abfc9b73e7a1f8cdbc3b1a83d27fa04773f2872d103c3efdc48b11493e208cccb0d739dcb5aa7d560e4b29eb5a356c45e3c1efde27f19c6ff916a6a753c199a56ab9763121ce59fd66ff64debf7c5b83dda49949dbc0d2a60fcebd30e1561321ee3ce3f16af449c58dffe94d80a4c9a6365d2cc70d8bb3b29b3a07bf8624cd2e364924e71a49b0b596db7e6c24b9add73bb1293f07877737a76bbbd8acb4aaf37359b54259854273869c3a12e35c523e487b55f8a4970a24c82d98de272a3b38216df3378144571c2d83feff50dd27323731926f349586d93eb228d9d5d495eb2659fc3e653e3b725724a7ecfa1aec2243cd95d0ca05cc23dd9f8adca41dc6250f29663f29e229337dfa5bcd359dd4c3c171b8cacab31594fc53cd2ea6bac3ff5b20293ec34996413124a66456cd258bf4ac26f2d29153a375b2fb529f138dca6f7ea6b30794e775252b1f9c29e9dea59319b6ba1f1a8e571e2306369f827d95a4cb233b0584da7fd5ee2b86295b12795985c5b9cadc4f1bf6e416fa63fb56c1d26e399586c87f378a8e68ac3acc2643a0bb3c39c72112b514943c4a5dfb3ead5980467bbb3493009ec48529bffa6cc357a2ba31a4cc273dc8d001295113de4b784112a14e23754cc81ec8749752ed66779956a214fd5cce9d462f29c8745777895673adf1f83c6efda5e823a25b2d561b29d2f936d7edcb852eb28697c2d62bc28fce8f74d99af8b7eed46f67a4cf60b64b20f732cbb5d9eb5be797dd6639c5ee8ce87257a0a16f016bb14f849ddf8a99bd6ae6fc0a4b9c81fdccb7dc37d23c6e9c5fe70bad2374e37619c5e22e374af66f659f8376a6a4d7ec261ba35df74a68b740450c8ac5c6b4d48069d8c0d6782d36bec88f0f498e8f1e87d9aff3ac2132db1450db1dfd1d36c300a9712a05b45f80696c3c3b490ec034524fb47591a51c9e5b6fa62ba3e229fcd5d5b2bb54ac8b53d532c9e388449218b824f0771f050bac432d9bb4889bb7687c3adb944362d82749a7ca72cb325bd8c966a5b5eaa918c1154c6514dd4a6c45c5c4e736b21d7a1c67babc3da21e3dd22308f8f49c25dd92137349b421622812f084c98b3b8e79798f31846c08e544c486e31e84864e47a3375b302ae6b79e8e873ad42a1d649914c5a225b55c2d611c016dd6f65166f63e82a5713e0a2db13da705da15ecb1a2f92a5464621aba86bc9ef7227726a00dbeaa4286c57aa84ad67839645f78ba96323d39a98714319f37d98a2755703adbb4a257c298c403d7301d633523470c10488b646a2105dad1222b67cb36115479921cd3cfd67c3738d4a783ab10a5814ca6c58e45fce0d125ae50a8d016c3b00ec5a95807503fa44d95ccbec6e6ae7cc23bfa395d094b66a2b00ee3a95c0756968eaa7b1ac4db6c2dab58e8276bd4ad03aa3da669a9ed5033dbb417505b50dd7ea049b959800dc1ca66bad9304e2337c1b71ba431b1f1b7764de884d6cad642da3572a8e95e986cada717e231ceba6490c863487d530cd55e790e6a8ef4d7330104b6b82199d86d88e692e4632692ec6d6698edbff34573538ea08933bb032a923eca6ff711f5526f551dd4c41a0bbb432970e6b876f51d90e0f8cd16e2bb5a71e8bf927b5a024a90bb89c853354b2f76bd9bb02577d13ec486fb7c432fe5262daeafe4e446e4a7bbd1be8e86e55efa129a2b1be247b6b854d116bb48af94e1ecac0eba90abc4c00de6d2ac1ebc5c1b31ce6554ce7e285cb64113194c1d95d159c2f01386fd709ce6c23e16c0bfc3a77040167390ae8e298308dac509e3a385354c1390768e79d3ac19963249c870138efd209ce6c23e1ac00706e5309678f046d676c451fab0ecaaeaaa07c0b4079b72e50e61809e568d005dda30b94d94642d90e4079af4a28d3389495e4a54a36b2b5c7e805841d18c4e6741bd64c03c745cf8046e0bea0a634ecdd673605353252a6ad9b017cf73767c58f0d83ca5207656b5550ee042afe03418cd0f3044003d0c341aac03b01e8e18341b69a9560db4eb81da4f38d2315b69a3fa982fb5500f743cd5bed613cac4228c7ab82f27c00e5c32aa1dc298ee6c633edcdeb94be04903ed29cda191b4dab503b8b5441792980f2519550d24307ad8518256cc6c69e0132cf1d970c207b2c08ed2be17341f5714da24c034da25400e5e32aa1ec2fa9c84e970d2985b6bd2a686f04d03ed1bc5a6a7a0f3e1040f964737637b11b28299ccae8125604e50f00caa7823086e03470745dd0141ef450c5a6eccb8398f855d6861e0d207d5a25a47d1db7a1a2df28cf40bf511218ad3fa33ad6ae88c574c43bdbcb3c33a90dd0cb67f59a53cf3310ce77019ccfe935a76e229cdd41f57e5ef542802ae60aaee7b179cac0eaab0aacf640f75e50bdb0c40af9836159e6751e1d01602faa5fc8e4d47d9665a4f93d0a54d597829ac1890dc50dc03e6cab0abeae40175fd627a020db40bfe31300ca57f409283011ca41a052bfaa4f40818950fe0eb4727b104e347b82a1b4617f7d27130d990642fb1780f63595d00e16a0ad6096a2dbe3479582acccbabc0e80fcba0920671908720fd0debea1cf84ee7803edd1e100ca378330ee9d06bc4e3030dc753ca8fc6fe904e67803c11c07c07c5b2730c71a08663a00f31d9dc0cc3510cc3100cc777502739c8160660330dfd3094c13d75b640030dfd709cc2c03c1ec07c0fc402730330d04b33700f3439dc0cc3110cc9100cc8f740233db40308701303f0e024c2cb6489c8bcc347026720800f3139dc0cc3210cc5d00989f063557b421e6a4f1005659289b2bea0ee0fb4c257c0304575c3ddba786c2b8e33a936f06e07e1e840b2e53d8edbe9c6f97b38a7cb4c9e898b78301945fa884b2b7c4656c2fecdb9156547c0100fd5237404dacecf70140ffa5cf2281b10656f64200e5572aa1a4dd6a259ff5a923e6d07ad2525a9b972903adb32ad05a01d0be0e625218ea9f7c6d64744d64745593c2aabd4115b47b0168bfd133e07aac8131737f0358bfd56b9723131d1e53019cffd66b972313e1dc00e0fc4eaf5d8e4c84732580f37bd5f6a6b32d3d1730d6ac13b49401ba5815a03b03407f086273899ce0379750d6f5cc02e0fda812bc3e8e7bf4d895e50a9dc493552df5eb07c2647ed211d42c0341ed0340fd594750330d04b53700f597a057a13bdf4726cbc07d645e002debaf2aa11d86c426ffef6cd9038f22f9ad39e791c49dbaf20cf439d502307fd709cc710682792000f30f9dc0cc3510cc1a00e69f3a8169a26b793f00e65f3a81996d2098ab0098ffd109cc1c03c12c0760fead1398266e33550ac0fcaf4e609ab88d691900f31f9dc01c6f2098c7023043118dc09c602098270330c3116dd6b88d33708ddb3f00ca884a28db7328adb8242b14cca25da172d7a32455c0c113545b449ac597641fdc65a774b3b3d5e4b30d6ca2633d5bc29a6d6070624be0a66b19d1661bbe5c03b7e10b01289322cd189d64fe2a8d6ea0c2b75209e510c4c3e92c2664323b82911ee3a814666527b43d0b606e1dd16a373913e3e46f03702647b4da4dce44387f0270a6e804678e91700e06dd539b8856a160262ebe9e06b4b36d44ab503013e1dc1fc0d92ea25528988970960038db470208a785b6e88e7beed516006b878836dbfe8d33702dc76700ca8e116df6aa3311ca8ea067ef1409204c3173073aa53a0fe861e748801ba551082bd82714bcd21dbadd7c0a80dca539c6f37318d4f509e395a2ef2984f9755530df0260ee1a54431070bcb2b2f1513e00af5b24c0a83a78a4d8ff869ede0aa0eeaecf9c52b681734a1702287b44141f35265bf16ede8c66170059cf883607dd651b78d0dd6b00ca5efa4ecc65193831773f80b677449bb5ee26c6d08d0650ee14d1f640bc6c030fc41b0a869e7d22da1c8867624fbe0fd0d2befaccc467193813ff0d80b29f5ef39a130c1c197d0ae0ecafd7bca689700e006de68088965b5d98e863ba0468696a24806dd612c39acf3f096c3567a62a709f04e00ed4ab45cd35b0097813c03948af16d54438db8316757024c08dd64ac8509356ebb53bd8466bdf02fd1ca2cf3c67ae8150be01a01c1ad1e6783313a11c09aaf9307da68c4d84f2bf402b87eb53c1271808e52f00ca347db4d244283b830a3ea239bd498119ecca3cf35380568e8c04b0a78d35156ca5f96c07e44236af59409882c3cb0076485636c45c01401da57ab2cd02a9969099ca1c72552a215356a5db01c84637ef542f9cc63071c1752fd03a8e510965b704ad63203bcd2e5305e3ee4023d3f5093ec83270cae20200654610f146b93b50e06131002f5325785d410f6df5ccf3d8def0f58ce9b541f4c8ca4e609d0e40ccd2279623cb408bf1410065b63e010726eecdb50040991354bbb8a3ec1b3b1b8097ab4f488189fdf32000e5587d420ab20d0c29f81e4099a7e7a4ed38032b7b0a18d38cd36b51aa894785ee0ab474bc5e8b524d84b31ec03941af45a926c2b91cc0b9b35e4104e30c9cf5fe08c0b98b5e410426c2d9137445bbaa847368a3d3da4a43331980a5ec1c0d3731c39906c60c0f0140efd69c067e6c7f6f629cc669a00998a812cae1e88164f6ba0b3a49b18efcd0bfa9d6ae6724a78736b059a07a8205452d00fffb2eca365001604fd267cedcc493311f0750eeae4f508c89507603ade9647dc20f4c84f237a09553548ff1693b393fb491f5ecb5e4f5d5e4c59a983ebe8ced2e5746182c2464e82e734a4fcd1cad0ad6fe00d6a9fa4cb84d30d0a1370254f669aaa780ad13dea29a378bdcb12c51d3357227a091d355c23850a291d6541b5d985a4520a56652cd0eb77f6704003c439fb924138f26990fa0ccd7cb433ac140cddc1bc039532f0fa989709e04e02cd0cb436a229c5500ce5911c5a7b2af247f587052c3a89ca4f4fc56f3e6903a00d06647b43d953dcbc053d9970268e7e8d5f9641b58bd270138e7ead5f99808e77a00e73cbd3a1f13e12c0270ce570967f7461320f1fc9be6f991fe004016ea02a489d31b2f0320f7880474fefa8e125fbc0980b720c8fd0c2b585471e3753f3beed6134f03a817aa84ba33b9a01b3f16b00dcfe8b171945c15f9640ac16415771a9b37f7de1600b8481f1771ae812ee24701948b836833b3828f3d56b6cdc94c00de12f54bac6af8c9e8d61a8c29e421dd4ac2223283a4652c26c4bcbd0b5b0018f7d4675546ae81ab323e0050eea5ef0e9b39066a6932984cdb5b9f490a130f095d04b4741f3da3e3b30db42b1f03b02ed52b6036db402bf34500e732bd02664d84f34f00e772bdce673311ce61a03bda572f5ffb3803bd9973817616e9e56b3711ce23019c2bf4f2b59b08e75a0067b13eb1c479063a893f0450aed42700d644283b802ea82408df51e303eaabf82be68d81c2400f4b55823752f0b7af017199d3fea7ceafde06202f0b62d859eec8d31410ac37ab82f54400ebaa20e62f9d754ee6b5a7e70120cb7501d2c489e0730090ab750132c74020ef004056e80264b681405e01805ca30b90630d04f22e00e45a5d8034714be73b019095ba0069e271caf70020d7e902a489c3c9bb019055ba0069e2bed80f0120ab750172bc8140de0b80acd1673762130f55ec0b7c45fba9847227b07d69395b3b59c4372e2de4f33e760c5c193b455df966a6ca3646e901b4b35625a4ed40e856755081aeca42640e05b0d505015be60e01db0100b6faa0602b090ab641aa603b06c0b65e9f50d51c0343551f06506e084203b377888a7b04806da34ad8dab36873aca735af7f6d0980db1484c962cfd054b289ad724264f50ee6987d0e40ba599f48df1c03abf57b00cafd5542d9536250d7818a3d0d2c80526c4a2b9b191c0ec03c20881d5eecaa5e0796d23b0d4dcd323034f56300f081ba036ce29ab2ab00c007e9139d9e65a0ab621480f2e0a0d75038dfcb35cb40936a20f0021da2cf610d268e86ce045a7aa83e8735641a7858c3d700cac38288631b1bfc1a4865ade5f100bcc3f55ccb63a2c1f40080f508bdd6f28c3570b5c43b00ce23f55acb63229c5d403f7e94ea8dc7ac8d6de92b666f3cd609e8e0d1fa04f19b1897f63a80f2187d36043711ca3450958fd5673d848950fe0768e571fa547013232f7e04501eaf8f569a08652750c14f5009e510c1f556c2367ba8e5077ad873410b99b3b89e0d1677a4a98df781c69ea8da24927be0cdf3bba700d04e0a72d3b13af6b9b5ebeaffc6e2a70868064e0e7297c1aa1daca2bf0274f6147d3ced7906fa8ef600509eaa12ca641ae44e1e97b373a72ad98be6c5127506709da6a7ab2dd7c071f97700d6d3f5da49c3c4c33c260038cfd06b270d13e1ac03706ed16b270d13e15c06e03cb37936c81377ff6f7c0a8052c3e82755d03e0fa03d4b1ff7878936e62300cab3f5f16f9a08655730ee39471f4f928950fe0ab4f25c95508e8913100b2dceba268cddb30c1cbbff0c803fcf54e04d749adc0e803f5ffd06cf5690ad9de6b360da00c36b958db03602182f0822c0267b073a98612200ef4295e0f593047ad3d303e6b1bf37b20fd712a2f4aa9e350d81ac9f5416c4900580bd2888a1403d3bb5176f3bf3f907b18b13320ddc18bf2f80f66295d076024b5337b0b890f5cca36f76954f03f05d12c4da0e672ee66c234f443d0b8079a94e609ab88cff7400e6653a81996320986700302fd709cc4c03c13c0580b95527304d5c02732a00f30a7d56ade719b84ee3df00ca2bf559f29a67e092d736c09577953e4b5ec61ab8e4250ca0bc5aaf39cff106ce2a2d0695fc1abde63c4d84f33800e7b57acd799a08e73a00e77541b9df025edfa6cc775900c0bb5eaf855899063add6e0070dea0d7422c13e1fc1cc079a35e872a9908672a308b6e0a629b0ab71176a6cf64de04f4f566dd013671c6f22200f02dba039c6320c0ad411371abee00671b08700b00f06d7a0d4dc71a68fccf000dc2ed7a0d4d4d84f32000e71d7a0d4d4d84733580f3ce205625615e7bd3f77caf0640dea50b90262e97ad04406ed30548134fb31a0a80bc5b17204d0cbbed0380bc4717204ddc9f613200f25e5d8034f134ab1100c8fb7401d2c4d3ac960020efd70548134fb39a07807c4017204d3ccd6a0d00f2415d8034f134ab3d01900fa95edb6e6dbb4f992c25b7361908576b00d7c341846f399dc8186fe034c6b500cc477402738281605e0fc07c5427304d9c603b1b80f9984e60661908e6b900ccc7750233db40302f03603ea11398390682793900f3499dc0cc3510ccad00cca77402d3c49d8baf04603ead1398790682793500f3199dc01c672098d700309f55bd6632f69cb679e477bde1a7b52501f89e53095f07b04cba866fd713c872d3b6aaa0eb05a07b3ee815e6050934d1e4137a7b02605fd06719d558039751fd0b40f9a23ecba8c61ab88caa1508b07a29c84d77add6b280b00223d676e453d9de065afb72f3ec36b798b0433dec763a85bcbe9a7cb681bc5bcd76e8560aadb24eeb6200ed2bfa6cc36be23af45c00e5ab41f4ffd18d509ceb688e813aba2f0076bbaec0661b08ec5800ec6bba029b6520b03900d8d77505d6c4ee6a0000f60d5d81cd3310d8a300b06fea0aec5803813d1c00fb96aec0e61a08ec2100d8b7f5b15cc71b68b92e0450bea3d7e26c13e3463e0170beabd7e26c13e1ec0f1c2defe9795e478e814e95af9896b64e1288cf6844dc3aeba5928b5cc15bc174b6b16e153b56b4823c490fcd61dd4e7d6805a9b64bc98fbbb2584e7e64f7c36c3b8ce9a14d843d7ab79eb9b329ed75e429e5ac2e943992bcf23e558d08578db470fb46d333d01d9ee668fa86c2130e77e688d0bb73990bbdb86157dc45e4bfb58151063def3e9c477e59dd6e7a43876b21b88efc5857f6fdc434291ef1a97d40850ecbeb43a56f259948224bf9e8f7ab08ef254cc1e8dd449226960fd78a76714a2563342d8b1d59fe0f61b94732e886e3e12c1f742f31471fc16ad66c87f2e957b685ec20bc1af62665d38b6eb721d2c5d2ab60f966a4eba9d7fec9fe71ac5e673455afdd73f4894abdb6a859e3328176be03e9e4e520ca1c7dab86e94371832e58b57c35f9bbd40112fdd88807a736995c9533592b18d57521fb1fed2fad9f9eac2a2f26df96f2634ae793f74b193dfa2fe9ff00a0ba4e7db5a90200
```
