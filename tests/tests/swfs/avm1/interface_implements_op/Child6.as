// Compile with:
//  mtasc -main -header 200:150:30 Child6.as -swf child6.swf -version 6
class Child6 {

  static function main(current) {
    // exports for test.swf
    current.isInstanceOf = Child6.isInstanceOf;
  }

  static function isInstanceOf(obj, cls) {
    return obj instanceof cls;
  }
}
