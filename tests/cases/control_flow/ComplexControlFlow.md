```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class ComplexControlFlow : UdonSharpBehaviour
{
    public int limit = 5;
    public int seed = 3;

    public void Start()
    {
        int i = 0;
        int j = 0;
        int sum = 0;
        bool flag = false;

        do
        {
            int local = i * seed;

            if (local % 2 == 0 && i < limit)
            {
                sum += local;
            }
            else if (local % 3 == 0 || i == 0)
            {
                sum -= local;
            }
            else
            {
                sum += 1;
            }

            j = 0;
            while (j < 4)
            {
                if (j == 2)
                {
                    j++;
                    continue;
                }

                sum += j;

                if (sum > 50)
                {
                    flag = true;
                    break;
                }

                j++;
            }

            if (flag)
            {
                break;
            }

            switch (i)
            {
                case 0:
                    sum += 10;
                    break;
                case 1:
                case 2:
                    sum += 20;
                    break;
                default:
                    sum += 30;
                    break;
            }

            for (int k = 0; k < 3; k++)
            {
                if (k == 1)
                {
                    continue;
                }
                sum += k;
            }

            i++;
        } while (i < limit);

        if (sum > 0 ? true : false)
        {
            Debug.Log("Positive sum");
        }
        else
        {
            Debug.Log("Non-positive sum");
        }

        if (sum > 100)
        {
            Debug.Log("Large sum");
            return;
        }

        Debug.Log(sum);
    }
}
```

```hex
1f8b0800000000000003ed9cdb721bc711867b49002478a644123c89271d488a960051b22c27716c49a4643ad4213cc58962531009da50408a06e844aeca13b85c295fe601f21279035fe6017c979b5cb97ce90b5725ddbdb3e66076068b2530e4023150c4627767fee9fea6e7b03b0bb6a4015fce1bf8b1056b701faec326ecc22b38c06ff771bb8f6f77cf3bfe148ab8fd043fb378ee0d980ac849af5967183f5770af044798f373d8c16d9ecfaf430ebfaf60de1ceed1d13dfccb612a2786993661099ec06348398301125b9ca9248e39e497d38e9ff7e00b3c9b63a3a890349f4ae1c73a9e29f1b97d34da4bf70c3e62b7f6f1dc0ee6294201355fb062fb5709e02fe06ee2629b105bb231a91c1bd21c4b49c7dac476583a3628b631b1bd20f25e94f6d91eb19d10e72f49fb727923e2fb65699fb4ffa394d7aaa4bf529ede9992ec97ed4889f4b32e13e77be5bce7d7a462e7a462e7a862e7a828f7b5d0fdbb626f50be7f05d83ba7d8eba5ebd0f865aad321b1ed14db3185dd986b4beb9492deabbb71c5f671377dcb574afa0e695f62dde2c5484a49afdaee9def12db29717e5edaa7727f10fbdd501e97718d0f41360d0bdfefb8c7e37f35d49f8141ebd78aed3d1adf5037f65cd1f5ec9a5274858faddf887ca0e4f3ea645ac937ede68b7da8e4f3eceaad60575ce493d3f705a4cf287c66e038de66a0bc4d79ecae4019bbf825257d0093d8b7a2ec1f155b67c06fab4e7746f1e5c85027838a4e957d0fc70ef5b72a4bb50f9897f629df37501ecf63f0533cc7bf85f23ed83b27b5db8417cbfd42ffaa9b3731208e9f938ecb769d57ea671ecaea27f1a9d81f50f2b7081bdac57799bd364ddce191f27d1cbbb2700869ce44a5553ba47b39830773924ed007bd66b9d8231c360f514033a26ef2787d043761513ba28e385d92ddf7f9338b29f22cbae02fe29fbe22c8b802be73d274a084c71ff26c822604793c731d56714bb99ec30d1ce09f292a5b586c01a71439d8c0cf43d67b8e66fb5306b944530735cf1a6aba339b7d3ce2013dc4fd025bb8ce9f7f664b736cfd0a1e3be279d601ceb4eee1f63597e6f7bd88df75656e08647a1bf5131cd371771ae555d3127a43d6a7b9bdfe5c23675123843eb1209a612ccd3dd497be9af899a1dbc9d0a0b122346e3032a7ecd822a4791ebfe62318de47bf87faf87679df865b157dd193a1ea1e630f8e6bf8bb7f7ffdf1cb9ebffd235eeeda4d48f3f0381ee05a25b0a437e70cf9142abb108fc73264a88adfd1e1e7e177fd14f1bbe9f29c2e7c05f0ec45a90087a6019e8164de6bde3b100615e001febd82bf80af863209310199d30ca141869a30b7e830b79f36e6e07ea512e6f69ffc3ac64c2f3dc1a48160900d2682ad3a82c9b3e9274ec68f26dc29855fdcc8afb362330f4b2fa6a197217a5d1aa308753ca91845f618c4e33a718a966e4dc4fc175f3ef1a4593c61b2bc47633984b4bc4d17547c61b4718a4145e75eb14a16d3860f2cbada5d5049e8a3aa8fa36ad8e75cb009a6c86a37d54fbfa67e5a42d64fd2247eae0e61db61123faf1177428a779ac40734e2b190e25d26f1418df86248f16e9d38b11ad28499e353ee332bf798cc4e69ccee086976af497c58233e1052bccf243ea2119f0829deaf13a79e89ee98f8e63554f74fb15d96f8b23c8f1d4e0e5b6b89af027da5b6994b3d672a754c572a0178cc5dd935ecf26a2ffdbc09e8b806e86e48a00326d72ee85ca3525771b788bdf4c99c19343933a17106423a3364129fac8378ca243e5507f161534732ade948fcca153a921193f24cadcaa326e58bb52a8f99942fd5aa3c6e52be5cabf20553785ca943784c98c467eb203e69129fab83f89409f87cadc0a74dca576b559e310159a803908b3a71ea306921dbd7e12e43f9ecf3f88ae63a6ce3fb158e30dbf0081317788c39e419e80e0a78f72f29955ec17cdc74264c477fc9e4e7359d9fef55e5e71a9fcbf23d055a0e2f9e9977974dde5dd77977b72aef96e1334c9ee51aa46583da7d2bbf5209e3df15937fe993fbb70a74a957820df894ed394bff664dfe6574febd5b957f77312677c55cef2c5bde9cc9b71b3adfee55e5db3a267e0147fce4ccce997b386ff27051e7e152551e3ee4db03594e538c448c5e35797953e72505ee2696e0f61dcb7c2325cfab564bf8f702137d22fc5de567a0543b9f609a97e0ae7ca9e7b630471e635b6f7d1bbe1d8786bc6571cff80bbeeacaf35e09d27c7d9f81ea1775a9ec7db487b86da08b2f985ff012ef2d302ff1ee41bd56fb82bc706381f2ef716bf1569074de05fb645ed96c131e795a489a6f76349ba76f52ad3aa2565bd27ccf65e184d1146cc56d2aad551f43e1d7385c81c7fcbc210ad06dc56dce4ed7d047c1fa72abd4e95f63016f09ac0e4f355050b90359510cd7f4a226ae883fa85b949d6c79cbdf1e7ab9077acdf75c8aacb7abb60fbeab1635cbdf92e37bce1723c7ddabbe5bccd0a93bffe73d5f867abeb709c245afa3c85047f10b9bad79886b6a9b93b8cff6960f9bb2dc36ce2c95f26b59d84e96b5fbb66a9c3b49dbefe7b6efa2fca54d9483026501131570fbb2c2fcca22c84e5b20872490bfb20932a58074efd03617cc9404f39dd380696ae0e5737bab30fb6cc16c9360fe3a4a306f3420cc6e09e6bb36618e2acd7c0f3fb33ccc345b7c0e4b48dfb3897444414a8f3cedf0f57573f59e1724a0776d02ed1640a9e03dc6e869d3bd046be0e2b6c08104ee9e4d7041f3cacdd38ac1a42d947109e5fdb344593e455f6cc029fa808472c9264a758afea726eb17672490cb3641d24e09bfe46cf682d630c5244c0f6c629a16f1e6de463ee09823238e904811f73f103fe668b6beb14502fcf0341b74bec91af4a004f27d9b207bc134d13910491a6f587124782ba7118572333745a1c54b426b51382981fc202a201bb1394f48207f631364b839a3c598b4d6b8cf4928576da2a4797e81fda665e9c68bb95609d4a3d368bca698931befdb0d08f2bc04f2715440de6940907d12c8275101d988e372bb04f269544036e2b89c9040fe362a206f3620c80e09e45a54405abc25660d645202b91e15906f3620c82e09e4465440de6a40909d12c8cda8807cab0141f64a20b7a202f2760382ec9140fece26486fd9df745ba219d6a84724981f460966233e43312ac1fc7d94605a9c0a59833926c1fc4394605a9ca05b83392ec17c16259816a744d6604e4930ff1825981627ead6604e43d33ca39fa199c947203dce3ea7dc469e2b5b41771f8fef179e1e60798f801689b3782ecbdbe347e5d3fc8f2fe8171e617e3b525933f839f68f417abe5fa9a142dd6aa81ecfe6eb3cade539fdae0ab592e6ff13d2ccfe6f83fb8f4ff945619ae25f716c62cdef8a1f1a3ec1c4ee0f99e915ff1fedea51f2d35c0000
```
