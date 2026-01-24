```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class SwitchCaseShort : UdonSharpBehaviour
{
    public void Run(int mode)
    {
        switch (mode)
        {
            case 0:
                Debug.Log("Mode Zero");
                break;
            case 1:
                Debug.Log("Mode One");
                break;
            default:
                Debug.Log("Default Mode");
                break;
        }
    }
}
```

```json
ERROR: Compile Failed
```
