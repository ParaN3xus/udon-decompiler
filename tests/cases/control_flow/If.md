```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class If : UdonSharpBehaviour
{
    public void Start()
    {
        int a = 10;
        if (a > 5)
        {
            Debug.Log("Condition is true");
            a = a * 2;
        }

        Debug.Log("Finished");
    }
}
```

```json
ERROR: Compile Failed
```
