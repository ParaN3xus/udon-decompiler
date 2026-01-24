```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class VariableScope : UdonSharpBehaviour
{
    public int fieldVal = 10;

    public void Start()
    {
        int rootVar = 100;

        if (fieldVal > 5)
        {
            int branchVar = fieldVal * 2;
            rootVar += branchVar;
        }
        else
        {
            int branchVar = 0;
            rootVar -= branchVar;
        }

        int i = 0;
        while (i < 5)
        {
            int loopInternal = i * 10;
            rootVar += loopInternal;

            if (loopInternal > 20)
            {
                int deepVar = 1;
                rootVar += deepVar;
            }

            i = i + 1;
        }

        fieldVal = rootVar;
    }
}
```

```json

```
