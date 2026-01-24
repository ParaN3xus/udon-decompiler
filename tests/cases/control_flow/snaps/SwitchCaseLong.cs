// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class SwitchCaseLong : UdonSharpBehaviour
{
    System.Int32 __0_id__param = 0;

    public void __0_Run()
    {
        System.String __lcl_itemName_SystemString_0;
        __lcl_itemName_SystemString_0 = "";
        if (__0_id__param >= 0)
        {
            if (__0_id__param <= 110)
            {
                switch (__0_id__param)
                {
                    case 101:
                        __lcl_itemName_SystemString_0 = "Apple";
                        break;
                    case 102:
                        __lcl_itemName_SystemString_0 = "Banana";
                        break;
                    case 103:
                        __lcl_itemName_SystemString_0 = "Cherry";
                        break;
                    case 104:
                        __lcl_itemName_SystemString_0 = "Date";
                        break;
                    case 105:
                        __lcl_itemName_SystemString_0 = "Elderberry";
                        break;
                    case 106:
                        __lcl_itemName_SystemString_0 = "Fig";
                        break;
                    case 107:
                        __lcl_itemName_SystemString_0 = "Grape";
                        break;
                    case 108:
                        __lcl_itemName_SystemString_0 = "Honeydew";
                        break;
                    case 109:
                        __lcl_itemName_SystemString_0 = "Pear";
                        break;
                    case 110:
                        __lcl_itemName_SystemString_0 = "Peach";
                        break;
                    default:
                        __lcl_itemName_SystemString_0 = "Unknown";
                        break;
                }
            }
            else
            {
            }
            UnityEngine.Debug.Log(__lcl_itemName_SystemString_0);
            return;
        }
        else
        {
        }
    }
}