package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var testcases = [
    '<root a="" a="" />',
    '<root a = "" a = "" />',
    '<root a="" b="" a="" />',
    '<root a   = "" b="" a  = "" />',
    '<root a   = "" b="" a  =  "" />',
    '<root abc="" abc="">',
    '<root a="" a=""',
];

for each (var xml in testcases) {
    trace(xml);
    try {
        new XML(xml);
    } catch (e) {
        trace(e);
    }
}
