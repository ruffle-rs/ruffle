package 
{
	import flash.utils.ByteArray;
	public class Test
	{
		var TESTS = [
		[6], // UNDEFINED
		[5], // NULL
		[1,0], // FALSE
		[1,1], // TRUE
		[0,63,240,0,0,0,0,0,0], // INTEGER
		[0,63,241,153,153,153,153,153,154], // NUMBER
		[0,127,240,0,0,0,0,0,0], // INFINITY
		[0,255,240,0,0,0,0,0,0], // NEG INFINITY
		[0,255,248,0,0,0,0,0,0], // NAN
		[2,0,4,116,101,115,116], // STRING
		[8,0,0,0,2,0,1,48,0,63,240,0,0,0,0,0,0,0,1,49,0,64,0,0,0,0,0,0,0,0,0,9], // DENSE ARRAY
		[8,0,0,0,9,0,1,48,0,63,240,0,0,0,0,0,0,0,1,49,0,64,0,0,0,0,0,0,0,0,1,56,0,64,8,0,0,0,0,0,0,0,0,9], // ARRAY WITH HOLES
		[8,0,0,0,2,0,1,48,0,63,240,0,0,0,0,0,0,0,1,49,0,64,0,0,0,0,0,0,0,0,5,104,101,108,108,111,0,64,8,0,0,0,0,0,0,0,0,9], // ARRAY WITH ELEMENTS
		[8,0,0,0,9,0,1,48,0,63,240,0,0,0,0,0,0,0,1,49,0,64,0,0,0,0,0,0,0,0,1,56,0,64,16,0,0,0,0,0,0,0,5,104,101,108,108,111,0,64,8,0,0,0,0,0,0,0,0,9], // ARRAY WITH HOLES AND ELEMENTS
		[8,0,0,0,3,0,1,48,8,0,0,0,2,0,1,48,0,63,240,0,0,0,0,0,0,0,1,49,0,64,0,0,0,0,0,0,0,0,0,9,0,1,49,8,0,0,0,2,0,1,48,0,64,8,0,0,0,0,0,0,0,1,49,0,64,16,0,0,0,0,0,0,0,0,9,0,1,50,8,0,0,0,2,0,1,48,0,64,20,0,0,0,0,0,0,0,1,49,8,0,0,0,2,0,1,48,0,64,24,0,0,0,0,0,0,0,1,49,0,64,28,0,0,0,0,0,0,0,0,9,0,0,9,0,0,9], // MULTI DIMENSIONAL ARRAY
		[3,0,4,116,101,115,116,2,0,5,104,101,108,108,111,0,0,9] // OBJECT
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
			ByteArray.defaultObjectEncoding = 0;
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
							trace(obj[prop])
						}
					}
					trace("done showing props");
				}
			}
		}
	}
}