```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class DoWhile : UdonSharpBehaviour
{
    public void Start()
    {
        int i = 0;

        do
        {
            Debug.Log(i);
            i += 2;
        } while (i < 10);

        Debug.Log("Loop finished");
    }
}
```

```json

```
