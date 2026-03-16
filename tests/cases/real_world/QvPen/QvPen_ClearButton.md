<!-- ci: skip-compile -->

```csharp
using UdonSharp;
using UnityEngine;
using UnityEngine.UI;
using VRC.Udon.Common;
using VRC.Udon.Common.Interfaces;
using Utilities = VRC.SDKBase.Utilities;

namespace QvPen.Udon.UI
{
    using QvPen.UdonScript;

    [AddComponentMenu("")]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_ClearButton : QvPen_PenCallbackListener
    {
        [SerializeField]
        private QvPen_PenManager penManager;

        [SerializeField]
        private Image ownTextImage;

        [SerializeField]
        private Image ownIndicator;

        [SerializeField]
        private Image allTextImage;

        [SerializeField]
        private Image allIndicator;

        private const float keepSeconds1 = 0.31f;
        private const float keepSeconds2 = 2f;

        private float targetTime1;
        private float targetTime2;

        private bool isInInteract;

        private bool isPickedUp;
        private const float keepSeconds_TextImage = 5f;
        private float targetTime_TextImage;

        private void Start()
        {
            SetActiveIndicator(false);
            SetActiveTextImage(false);

            penManager.Register(this);
        }

        public override void InputUse(bool value, UdonInputEventArgs args)
        {
            if (value)
                return;

            if (isInInteract && Time.time < targetTime1)
                UndoDraw();

            isInInteract = false;
            SetActiveIndicator(false);
        }

        public override void Interact()
        {
            isInInteract = true;
            SetActiveIndicator(true);

            targetTime1 = Time.time + keepSeconds1;
            targetTime2 = Time.time + keepSeconds2;
            targetTime_TextImage = Time.time + keepSeconds_TextImage;

            SendCustomEventDelayedSeconds(nameof(_LoopIndicator1), 0f);
            Enter_LoopTextImageActive();
        }

        public override void OnPenPickup()
        {
            isPickedUp = true;
            Enter_LoopTextImageActive();
        }

        public override void OnPenDrop()
        {
            isPickedUp = false;

            targetTime_TextImage = Time.time + keepSeconds_TextImage;
            Enter_LoopTextImageActive();
        }

        private void SetActiveIndicator(bool isActive)
        {
            if (Utilities.IsValid(ownIndicator))
            {
                ownIndicator.gameObject.SetActive(isActive);
                ownIndicator.fillAmount = 0f;
            }

            if (Utilities.IsValid(allIndicator))
            {
                allIndicator.gameObject.SetActive(isActive);
                allIndicator.fillAmount = 0f;
            }
        }

        private void SetActiveTextImage(bool isActive)
        {
            if (Utilities.IsValid(ownTextImage))
                ownTextImage.gameObject.SetActive(isActive);

            if (Utilities.IsValid(allTextImage))
                allTextImage.gameObject.SetActive(isActive);
        }

        private void SeValueIndicator(float ownIndicatorValue, float allIndicatorValue)
        {
            if (Utilities.IsValid(ownIndicator))
                ownIndicator.fillAmount = Mathf.Clamp01(ownIndicatorValue);

            if (Utilities.IsValid(allIndicator))
                allIndicator.fillAmount = Mathf.Clamp01(allIndicatorValue);
        }

        public void _LoopIndicator1()
        {
            if (!isInInteract)
                return;

            var time = Time.time;

            var leaveTime1 = targetTime1 - time;
            var leaveTime2 = targetTime2 - time;
            if (leaveTime1 <= 0f)
            {
                EraseOwnInk();

                SeValueIndicator(1f, 1f - leaveTime2 / keepSeconds2);

                SendCustomEventDelayedFrames(nameof(_LoopIndicator2), 0);

                return;
            }

            SeValueIndicator(1f - leaveTime1 / keepSeconds1, 1f - leaveTime2 / keepSeconds2);

            SendCustomEventDelayedFrames(nameof(_LoopIndicator1), 0);
        }

        public void _LoopIndicator2()
        {
            if (!isInInteract)
                return;

            var leaveTime2 = targetTime2 - Time.time;
            if (leaveTime2 <= 0f)
            {
                Clear();

                SeValueIndicator(0f, 0f);

                return;
            }

            SeValueIndicator(0f, 1f - leaveTime2 / keepSeconds2);

            SendCustomEventDelayedFrames(nameof(_LoopIndicator2), 0);
        }

        private bool isIn_LoopTextImageActive;

        private void Enter_LoopTextImageActive()
        {
            if (isIn_LoopTextImageActive)
                return;

            isIn_LoopTextImageActive = true;

            if (isPickedUp)
            {
                Exit_LoopTextImageActive();
                return;
            }

            SetActiveTextImage(true);

            _LoopTextImageActive();
        }

        private void Exit_LoopTextImageActive()
        {
            if (!isIn_LoopTextImageActive)
                return;

            isIn_LoopTextImageActive = false;

            SetActiveTextImage(false);
        }

        public void _LoopTextImageActive()
        {
            if (!isIn_LoopTextImageActive)
                return;

            if (isPickedUp)
            {
                Exit_LoopTextImageActive();
                return;
            }

            var time = Time.time;

            var leaveTime_TextImage = targetTime_TextImage - time;
            if (leaveTime_TextImage <= 0f)
            {
                Exit_LoopTextImageActive();
                return;
            }

            SendCustomEventDelayedSeconds(nameof(_LoopTextImageActive), leaveTime_TextImage / 2f);
        }

        private void EraseOwnInk()
        {
            if (Utilities.IsValid(penManager))
                penManager.EraseOwnInk();
        }

        private void UndoDraw()
        {
            if (Utilities.IsValid(penManager))
                penManager.UndoDraw();
        }

        private void Clear()
        {
            if (Utilities.IsValid(penManager))
                penManager.SendCustomNetworkEvent(NetworkEventTarget.All, nameof(QvPen_PenManager.Clear));
        }
    }
}
```

```hex
1f8b08000000000002ffed9d077c5cc959c0df4a5a75c93edbb2e56edfd9e7bdb32dadaa659fcbd9926ccb962c59eddc55d792ec5539159723c08592c6110281d4230408bd1d904042280784500347083dc0d10f08bdc34198f9de3cedb7b3f3bddda77d6f352322fda4b7af7df37dff69dfd4cdabb5d84fe810fb3768f55aad568d35608d5bb3d60cfbd4ca8ed3ecd73e73aef758f3ec38c1fe8fb07b87ac3d69dee43f0742d5ec7f073b5bb016d99b4bd6183b4ec1fd3e2bc63e77b07763ec8c5fbdcdfe62eca950017b69c06ab3baad4bd69650551a1183f0d282b816e276858ad9ff33d6437637064af1406ae1d616f6af8fdd59807bd34c69e7b9ebd64d306b9add1b63efcc5b7126731424167fa6c2820f967d58278eebc5b19cfd9588cf8fb0bfb065956e12e7f9e25eb1f88cdfdfa078ff0cfb2b647f77d0393f6e14c70a71ffae38dfe421bc2a29dccde259f67efe9cb8b6457aa63af14c415c5cdb2a8ead48ef0271dc86ee713de3d2fdede2fa7406faeeb0df0bbde0f26c9938b6215ddac47baf8af3768969bb38e2e7f9fd99e4f7f39cf8de65db5f512cb12943327667c0d6cdd6f50a997ba47b2a996725dbce8ae35e64234e2fe7a4e7cf89e3a3e2582a3d7f5e7afe7c9af4f89838ee13c7fde2f8b8786e569c1ff0906e0bc5b1437c9e43e72c9ef25f959ebb283e3f8bce1d7efcfabcf4fc7ef179419c3bf17c410aef821d5e418ff45ca6e11513e1b9d91e16c74e49974ea1cb27a4e7ba245dba503ec6ba1489e32549ee255b6e7887f45cb724b79b909b493cf64861f68830df21ce23e278593cb728bd7f9988af5e496eaf90fb9a387f421cfb24b9c5d2f54ce2a5cc965db8c3665f187579a75fca4325e87a086a3bfbfe12ca83f8fea0747f40ca5bcf88fbf7d039d3adc8b1ef495bc7f2f788f383482ecefb57849cfbe2fc10bace8f57253d0ea1f82a41cf38ac4559156e11d70e8b638d38d64a65c303178651c9f6bd2bd459b6f91af1fe3571bc2ebd7f5561f375179beba4f2d08bcd525a2b7e8b2dbbf84597776e4a69ad145de7c71b923d37a4b4744b4a4bb7ecb04b1c9ef5a22e74f46e90dec75cf6bb7051e9de28e9e4c4d110114743e2382cd9a4d265d84597a62cd2a513479536a75237fbd6a3674b90efc7df7b5e9c378b7af11517394724792a1fadc53e2febf1a0fb6b22ecd75dded9a0b0e168063e6306e1974d0a9ddfe0f28ee055f692383f9601af11294f54a0ebfc382aa59f51291d8d497962ccd6a1dc49af4f65a0c3a894a6c7a5347d5c1c4fa0fb2abfc98da3e363c7a47a3026f4fd9cf49c73bc8de2f2a407ffdc797f420a6fc2be577183086f1285776a05e14d49e14d89f05e969e7b5a1c4f8be71faac20887a0c5799e511a61126bb9d71f7ac243d3d879337da3988b2eccb3db91ac755c002ec843f66a4cd5321d8076ef222b61eb952dd3ada172a4772bfc1f614f4c81d0e75283f8584a105cb938fb8da166f502bb7e0e5ae5bc613dc5eed43067730ade1a66d5d975f69b2c6590051b679927c61c962516149737ccd44e7d329d49bc092ebfd3cb64da3d04d3ec8a03748e9dc741c33ef87f0f348d81f61decda22f457ccb0a478861d1f4068a9b6cfb3cfaa30fb0532b58e373d5db7bb239c686a63d670ed6b216d7f214656234638fac2e744362ca885d2e26d2931f105867621138686b62da3ceee024bba566fd542dbb33785a0771b532d54a76f9b77337318dd6c5193e1d1bd1d2c48c4f0890d9d9fdcf8a9f71f0c279bd660d5429373471ad3dcc0727991d0e61409ee2684c30551aea88c3fa4c20f4df4be1ce2b79f9b82e7bc474021d44cc91110da0dcdfa7b560f9473b802e5b9ab06dd1b6282e2509ace33259798f845783225eea285a2fb22a2a85cd3994045409e2a02a0ffa07f55d2fff534659f3a0a8a97ad4844412d7864d52e09950a0b5ca93ce14f395969f8ee3b3fbdaeeae31f09abe3a514e26525a1513193af8a99d25c678df475815bbc942ddb958817fea366584ea4ed743a50040b5404a17ffd46a004539d66e7d319f6ee2423710f3ca125f6becac97667ca1b7b8725a66a9e95c0738fd2f5f7a6114538ac22cca331341c28e10176cd6e16b4c3dd29f0259db2b583c91d6157ed0acced5977d2eb3226bd1e48ef83ee3c7f34a388172a8847cbc5385d8ab22592b25c4f42701125784396828b29c11bb3145ca24a7c15b9f71dec4a379ec6115527313e5ab43be302b28aacfcdd75a01253a92a6a2ac4408f522d397aaae8e82953454f65ae3d0b7e6f56f857332b88200efba04c421d3bd5840b905e052a7eca55f1532906ac53b59223a79a8e9c0a2ae6b7f910f39594dadbb3557b9d4a72a91859971d8dcfb39f14e1e5b4f0f594da3b156a87bca8fd08a5f62e85da8754b45dd4de40a9bd3b5bda1ba944b24799483ef8b49744b28962b257c1e4458f4caa28e18f2a847fd6a3f0cd14f0c7b205be85527b9fca8f0e7953bb9a527b7fb66a6f5595f49055df98633f5bdda17d0ee678c5ac6e56d4de113d914eb8b3e073718d9c1ecd5e76bc0d5af0eb63a24a750fcbbd0e795cd53a85d2a6dd43b77cf056f0292ca53c45106ddc0854707ae94c559ddb54a9bd488c97a6f4d66c84c1d7212bcafee2a227dae99fe6d7e7a07786db92922f8ae87cb19dd2e149950ee548074e6262598f794fa1eea0f2f9c16cf3f94e553e87ba6d568b7cce7b09e6a007ad9d49b6d3d669267382c9cf360ff35a395fcec350f53678c80fd969b8335402839fce9b4e37adf37310ee8f0a771345f2ee50298c8ac6d98d1198638a5dbb9d30a03309dee9788a50a22c380c6541ee6ca7f2f92eaacaac51559979deaaccdd94f05a85f0168fc2f750c2a30ae12f7814be9772ddea14ae5bef473f70d28bebf62825bc5edd78f0e4173ea612bece999823098fae17b36c44b5258713a1c3d94715ce4daac2791d14ce9d90b5e620298f43d135029df5f3bc2bcc4311bd9f02d8ec43ebeb7155290daefcccaa96d2bc8f6b099abe35a804e887b013e316d994d147a0304ba54777a8b44029569f4129e687f6541976802a098e2a4a82fe7c6f2541844a6bc77c486b4f50c29ff241f8931496e30a2c2f177ac37290127e4255b41779137e88c2725289e5f9535eb01ca60ac8535401f9f48a0ac81aaa803cbd9202b2de530159ab2ac336adcea8fcca46bef832972d1977ecb6ba0eaa7b1df78a5269bb4dd5dd54ec2d6dd751c9af9d4a7e675794fcea2923ce298c282ff166440325fcbc42789b47e18d94f00e85f0573c0a6fa2f05fa0f05f5c11fe66ca884e8511f1526f461ca1847729847fc8a3f0164af82585f00f7b147e9412dead10de58e64df8314a788faa4bd4a3f0a728e19715c2f7947b137e9c4a95bd54aaec5b51aa3c41d549fdaa3a6953529dd4cf2e3d80359ace18ec693127f19e6ae0d1a5763a496931a0d2a20cca46de7bb4003d61f7455bf8aea7304f51610eaac22c1623d1dc0b6d83b0ef7b0aed6955fd0b9deacfad6a1bc2ae0fb9bcdb3035d896770916cede879af16e92773e02fd0bfc6eb66d8b673cb72dae409d7e3283b6459056513ec2692a3d5d55a5a730f828ce2c352f29e90ce50a5ff3a185d0aa125e261630a59d5a50490b6ea3fa506f64db87da4ef1b8e9038fb394f05b3e083f47091ff241f8794af8b00fc23ba8d81cc936362f509247b3957c515502c3c8e39d559959a51ac9719bc3e4949b5db05dc11268e25ec28e653cdb6a1c4ad61a97d9567e684b959c9d549cc7b28df32e95e4cd623d53dae26c9c167c895279225b95bb299527b354b98752792a5b952f5345cd1d1f8a9a5e4aedbbd9aadd47a91df741ed7e4af8b40fc20728e1333e081fa484cffa20fc192a36e7b28dcd2b94dacffaa0f6554af8bc0fc2af51c2177c107e9d12bee883f01b94f0251f84dfa484dff341f82d2a1ddecf361d0e516a3ff041ed614af8431f848f50c29ff341f82825fc8b7c103e46c5e61bb28dcd714aed2ff641ed18a5f69764abf66daa19f7a55936e32628959fcf56e5494ae53766a9f214a5f29765abf21daac5ffe5aa167f84f4aefb97db197cb692dd0f31b4dcfae0d7b07f8fa7a67be937b84b69fb152a6d5ba536110ed5d693f7000e59a7591619079bec95c86edabaddf1c7c63865e357aa6c8c2df724393d3db89f47d587d4b1fc64a2f7a8175a4331d1eb392fe8d85bcf25ef8937087d3d53ecff28617562c561f21ddcea92df1a64214c31adbc709aa638bd49c56920604ebc57b59505b500239bd3e81d7742d97398a138bc59c5a12dc33cd1090c1658beb6a780f9932792173d78b17296b2f22d2a2bdfbe6ab1ddc607be98d487ec380ecf8e81d471f66ce6792599216d4962fe897af6891fe96b8e22ff5615f963cbe4fb188b8b2cc6ed118e01285de3a294b5fb9bb8761dec93fd3ad76bc84399b1f2b4f42c65d1db54161d27ebbcd439cbc9b5df84a2bf89df574b53f54e79b16a9eb2eaabbc59a5d203d707787ccc3d4efc487d0b94552fa8ac3a415ac5574f26c6f86c7b16443cdd86941967964dc36a5775a94d976a2bb16b91b2ebab557635937675c15c9a4966836d532b943e3c5546a12117bcbfb24459f27695256733ac7ffad8e3a3503e8ea03d6256d733bb4759fa352a4b2faeb0a6ed6636b7b3326a0944ad76bd7b9fb2f91dd978dc6d507e4c2d6f68bcbaf1fa80b2f16b5536bea0896f715678e4314f9e4562eed86afb150f29ea5fa7a2fea61c53a7c6d9875cb9a9f4c864c4deaf164211fb0d89b9253320e5216bbfcd82ac4526a316d612463daccfe0614fc3ea9138683cbabc9cdc5dc23b5d3681bbeddb7e602b9b43d1a1b42ef3b920a97b9f15098b1c598c342cac5c6b967e3d8fd56211ab79b5b0c4f3e00a53537a2dbe8187969fe94682e9f63ab2055c1265262c8aea862dff39f836e8e4984b1f8a3c762c8772180438cba57cd8fdd09e5bcd3ba8e685a3c0a7611585f3a29cfebb8244947e798b09b85e0f2770bd7b3571d51b816bb23881eb3d4167c0215133c596fd7c131075e62510bd374844858088bfce5d0533e0bc844aa7f70509c7cb3c5e13c09d2f4b807b7f90e0ca926abe1e28a3eec2e69f2660b250fa7a3137e593bd3e7900fa8b4c40341ce2888ac292f0b3016dc39be93e99a96eb3bd40e80174a1ce83bc71d98d861d1b74d3fc1bb11b1c71c95011c2cd8ca414ed1122bd45887a3292817716c9c02589782a4713ed4b75bb30ca13f007fe9f37fda2bce9f74d1cc21da7a504a5d507832cadaac53e211322a5cc40df21163990d4f7c3d772493a64b3ff6f4952a956eecfc6efa9255b0b2afcbf39489c9b054e0a667237643438945541a13c83507e4b2e508e89aceddedb1520caa2a0506e4328bf7575519a9e2a3722941f0ada095e443db0cefca57a03a1952268dfb61ad0ea0c845682a07d7b90d0b678aa4aea8cccb6d711ccefd009a68929f32682f99d3ac134b16c1c4230bf4b27980d06c21c4630bf5b27988d06c21c4730bf479fb64c8b8128af2294df9b8b7499692bfba8816dec8308e6f7e904b3c540984f2298dfaf13cc260361ee46307f4027988d06c2dc8560bea413cc2306c23c8060fea04e304dece4dd8360fe904e30eb0c84b909c1fc619d60460d84f90882f9619d60361808730b82f9119d60d61b08b30ac1fc915cc0a48620e415400166f3eaa060ae47307f5427987506c2dc80607e549f71c62603c7194f22941fd36b328189155023c2f96341e22c870daa92e7f538df711118b6c34161db89b07d3c486c0d8a0c1dc4d29b00aba82b4145025e03f0e34146424444429c3d1487630cbe1736b1e3c79062ced85a9b96308a70ff843e7df5470c447905a1fcc92051560894f3f0fd6171d88dc696cdd7cf05062e1c14383cf1f9a78204b74b910693cb5dfa6ba2036d3d5506857612a17d3948b495ae6932406fa03428747908dd4fafa67b6f7eefd23a84f26772e1de53954c6ab3d3c486e704c2f9b37ab5964c1c2f6a46383fa18f0b64e29ca4b308e5cf05ddf01c81d515a92e7a60d8d60785ad0861fba4be0e509d810ed06d84f6e7f5e9a4ab37b0936e1f42f90bfa2c0630b19c7c14a1fcc52051ee5664f8cc37470b14ee785070bb11dc5fd2196e9d8170bb10dc5fd6196ed440b81711dc5fd1c7133571a9413b42f9297deafd2306d6fba711ca5f0d1265298c262d2caf33e7396d2075f306035ae8e508d9afe9e32a999891f72294afe8d5d961e2acb97a84f3d773d3424fdd9cc1bc167a31c2f6e920b11df034349c7e37cb00dda096a0603f8e60ff46d0597e0a36d49c59e9864b0654469508e767f4f12a8f1a58195d43287f3348947b1528e7615607ef8a9bb12e40b69f5b732345f908f06fe9e3b6371ae8b69f40287f5b2fc7c9c4c5834f219cbfa38f4b6fe2fae04308e5efea95324d9c21df8070fe9e3ef5bb8929f33c42f9fb41b78e66adfb6276a7e9ada34284edb37acdefa837d0619f4238ff40aff2d1c425ac4711ce3f0c7e9b6cd51cb819f188795e6408c1fba320e16d14f0ea207b2fa47c19d31ccc6ce70f9b97a1ab11c4578384b843408cc2f7d1a8ab97c4578fe4086b60b5750461fd637d1c1f13f73f3a8750fe893eedee3a034bcced08e59fead350343155ee4728ff2c17954f740d563e9b11c43f0ffa6b0b547b680ee5728e666069b10261fc0b7d4ac866034bc85308e55fead5be31b1cf3c8a70be968b2959f48ac9fa35b73e7210c1fd2b9de19a38b7e30682fbd741c27d44b14edde92b4ab487ccabdb7720807f93ab862535ab63ed342c9f40583f17fcfe09f604840ef7ef0b33203596216c7f9b9bdef3b5b0fa278cb0fd9d3edd18cd0666dc3e84f2ef839eaa6a77ec76c150839df6e60d5c20558090fdc3ea3a38756bce7b1c4070ff51af968e897b0e1f4338ff49af71461377838b219cffac57ea347113e72308e7bf0489b32a4d0f518e2601b60605b21681fcd7d5fc3206399bb71898c96f2198ffa613cca306c21c4330ff5d279826563f6d08e67fe804d3c43d613a10ccffd409a68913b02e2098ffa513cc060361762298ffad13cc4603615e42305fd709669381307b10ccffd10966b381307b11ccffd509e61103613e83607e3e48985ba54eb9c5e5391b6b6ba4710401e5735ab5026a62cf663f021a0a693368d16420cacb08655e48dbdddfa2060e6eb422b4f9216d266d35183869ab06a12c0812e54e817291a5c029f89edbccd368838169b40e810deb0ad6c4bdaf0f23b085ba8235714fcdc710d8225dc19a585d6d45608b7505db6420d8e3086c89ae601b0d04db04608bc292f0b329c25ba14d1e17db334e0937a9c63a07936862c0788c9d7702b6456b98958cd7d9afb7d8b8c97e55d743b036b3dd7ac0d4e35717616f392e7b9adde59a2d58d16dec91529e38c2227144c8693e11d7296811d7e5dd11d7cdab23ae1b6771d42198cdca759f6177ba60a9c5087b6e048efc0b804681732d5f8814e29b9bdb49b06639f1d524ed2195b89e5e2667eb2ead2c6457b6aadc15f72d55a4b348f59549fc6a3a4bd3db47a7b0729758a9e559794ddb5fcee35d443b24d32d213e1d946fdf680be7fbb3ceb3cf5c29fe13fe3f70deec88c5fa0000
```
