package {
	public class Test {}
}

if(2 == "2")
{
   trace("2 == \"2\"");
}
if(2 == 2)
{
   trace("2 == 2");
}
if(2 == 5)
{
   trace("ERROR: 2 == 5");
}
if(true == true)
{
   trace("true == true");
}
if(false == false)
{
   trace("false == false");
}
if(true == false)
{
   trace("ERROR: true == false");
}
if(1 == true)
{
   trace("1 == true");
}
if(0 == false)
{
   trace("0 == false");
}
if("abc" == "abc")
{
   trace("\"abc\" == \"abc\"");
}
if(0 == undefined)
{
   trace("ERROR: 0 == undefined");
}
if(undefined == undefined)
{
   trace("undefined == undefined");
}
if(NaN == NaN)
{
   trace("ERROR: NaN == NaN");
}
if(undefined == NaN)
{
   trace("ERROR: undefined == NaN");
}
if(0 == null)
{
   trace("ERROR: 0 == null");
}
if(null == null)
{
   trace("null == null");
}
if(undefined == null)
{
   trace("undefined == null");
}
if(NaN == null)
{
   trace("ERROR: NaN == null");
}
