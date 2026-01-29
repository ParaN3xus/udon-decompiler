// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class HiddenInternalFunction : UdonSharpBehaviour
{
    System.Int32 __0___0_fibonacci__ret = 0;
    System.Int32 __0_n__param = 0;

    public void _start()
    {
        __0_n__param = 10;
        fibonacci();
        UnityEngine.Debug.Log(__0___0_fibonacci__ret.ToString());
        return;
    }

    void fibonacci()
    {
        System.Int32 __intnl_SystemInt32_2;
        if (__0_n__param <= 2)
        {
            __0___0_fibonacci__ret = 1;
            return;
        }
        else
        {
            __0_n__param = __0_n__param - 1;
            fibonacci();
            __0_n__param = __0_n__param - 2;
            __intnl_SystemInt32_2 = __0___0_fibonacci__ret;
            fibonacci();
            __0___0_fibonacci__ret = __intnl_SystemInt32_2 + __0___0_fibonacci__ret;
            return;
        }
        return;
    }
}