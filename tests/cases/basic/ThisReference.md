```csharp
using UdonSharp;
using UnityEngine;

public class ThisReference : UdonSharpBehaviour
{
    void Start()
    {
        UdonSharpBehaviour currentBehaviour = this;
        Debug.Log("Behaviour: " + currentBehaviour.name);

        GameObject currentGo = this.gameObject;
        Debug.Log("GameObject: " + currentGo.name);

        Transform currentTrans = this.transform;
        Debug.Log("Transform Position: " + currentTrans.position);
    }
}
```
