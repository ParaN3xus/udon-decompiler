```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class ExpressionInlineTest : UdonSharpBehaviour
{
    public void Start()
    {
        int a = 10;
        int b = 20;
        float f = 2.5f;

        int res1 = (a + b) * (b - a);
        float res2 = (a * 0.5f) + (f / 2.0f);
        Vector3 v = Vector3.up * f + new Vector3(1, 0, 0);
        if ((a > 5 && b < 30) || res1 == 0)
        {
            string n = this.transform.parent.name;
            Debug.Log(Mathf.Abs(f).ToString());
        }
    }
}
```

```json
ERROR: Compile Failed
```
