```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class Operators : UdonSharpBehaviour
{
    void Start()
    {
        // arithmetic
        int a = 10;
        int b = 3;
        int add = a + b;
        int sub = a - b;
        int mul = a * b;
        int div = a / b;
        int mod = a % b;
        int neg = -a;

        // logical
        bool flag = true;
        bool notFlag = !flag;

        // bits
        int bitNot = ~a;
        int bitAnd = a & b;
        int bitOr = a | b;
        int bitXor = a ^ b;
        int lShift = a << 1;
        int rShift = a >> 1;

        // compare
        bool eq = (a == b);
        bool neq = (a != b);
        bool gt = (a > b);
        bool gte = (a >= b);
        bool lt = (a < b);
        bool lte = (a <= b);
    }
}
```

```json
ERROR: Compile Failed
```
