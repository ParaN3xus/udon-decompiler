```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class ExternalCall : UdonSharpBehaviour
{
    void Start()
    {
        UnityEngine.Random.InitState(12345);
        int absValue = Mathf.Abs(-99);
        this.gameObject.SetActive(true);
        Transform child = this.transform.GetChild(0);
    }
}
```

```json
ERROR: Compile Failed
```
