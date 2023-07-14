package {
	public class Test {
		public function Test() {
			var one = new LevelOne();
			one.print();
			one.pubSameName = "pubSameName-fromTest";
			one.internalSameName = "internalSameName-fromTest";
			one.print();
			
			var two = new LevelTwo();
			two.print();
			two.pubSameName = "pubSameName-fromTest";
			two.internalSameName = "internalSameName-fromTest";
			two.print();
			
			var three = new LevelThree();
			three.print();
			three.pubSameName = "pubSameName-fromTest";
			three.internalSameName = "internalSameName-fromTest";
			three.print();
		}
	}
}