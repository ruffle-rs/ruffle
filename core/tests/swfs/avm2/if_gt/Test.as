package {
	public class Test {}
}

if(2 > "2")
{
   trace("ERROR: 2 > \"2\"");
}
if(2 > 2)
{
   trace("ERROR: 2 > 2");
}
if(2 > 5)
{
   trace("ERROR: 2 > 5");
}
if(true > true)
{
   trace("ERROR: true > true");
}
if(false > false)
{
   trace("ERROR: false > false");
}
if(true > false)
{
   trace("true > false");
}
if(1 > true)
{
   trace("ERROR: 1 > true");
}
if(0 > false)
{
   trace("ERROR: 0 > false");
}
if("abc" > "abc")
{
   trace("ERROR: \"abc\" > \"abc\"");
}
if(0 > undefined)
{
   trace("ERROR: 0 > undefined");
}
if(undefined > undefined)
{
   trace("ERROR: undefined > undefined");
}
if(NaN > NaN)
{
   trace("ERROR: NaN > NaN");
}
if(undefined > NaN)
{
   trace("ERROR: undefined > NaN");
}
if(0 > null)
{
   trace("ERROR: 0 > null");
}
if(null > null)
{
   trace("ERROR: null > null");
}
if(undefined > null)
{
   trace("ERROR: undefined > null");
}
if(NaN > null)
{
   trace("ERROR: NaN > null");
}
