```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class IfElse : UdonSharpBehaviour
{
    public void Start()
    {
        int score = 75;
        string result;

        if (score >= 60)
        {
            result = "Pass";
            Debug.Log("Good job");
        }
        else
        {
            result = "Fail";
            Debug.Log("Try again");
        }

        Debug.Log(result);
    }
}
```

```json
ERROR: Compile Failed
```
