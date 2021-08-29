package 
{
	import flash.utils.ByteArray;
	public class Test
	{
		var TESTS = [
		[0], // UNDEFINED
		[1], // NULL
		[2], // FALSE
		[3], // TRUE
		[4,1], // INTEGER
		[5,63,241,153,153,153,153,153,154], // NUMBER
		[5,127,240,0,0,0,0,0,0], // INFINITY
		[5,255,240,0,0,0,0,0,0], // NEG INFINITY
		[5,255,248,0,0,0,0,0,0], // NAN
		[6,23,84,101,115,116,32,115,116,114,105,110,103], // STRING
		[9,7,1,4,1,4,2,4,3], // DENSE ARRAY
		[9,7,5,50,48,4,2,1,4,1,4,2,4,3], // ARRAY WITH HOLES
		[9,7,11,104,101,108,108,111,4,5,1,4,1,4,2,4,3], // ARRAY WITH ELEMENTS
		[9,7,5,50,48,4,4,11,104,101,108,108,111,4,5,1,4,1,4,2,4,3], // ARRAY WITH HOLES AND ELEMENTS
		[9,7,1,9,5,1,4,5,4,3,9,5,1,4,7,4,2,9,5,1,4,7,9,5,1,4,8,4,2], // MULTI DIMENSIONAL ARRAY
		[10,11,1,9,116,101,115,116,6,0,1],
		[12,21,84,101,115,116,32,98,121,116,101,115] // BYTEARRAY
		];
		public function testToObject(arr)
		{
			var ba = new ByteArray();
			for (var i = 0; i < arr.length; i++)
			{
				ba.writeByte(arr[i]);
			}
			ba.position = 0;
			return ba.readObject();
		}
		public function Test()
		{
			for (var i = 0; i < TESTS.length; i++)
			{
				var obj = testToObject(TESTS[i]);
				trace(obj);
				if (obj is Object)
				{
					trace("showing props:");
					for (var prop in obj)
					{
						if (! (prop is int))
						{
							trace(prop);
							trace(obj[prop]);
						}
					}
					trace("done showing props");
				}
			}
		}
	}
}