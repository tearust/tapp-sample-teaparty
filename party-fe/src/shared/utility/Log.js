class Log {
  constructor(tag){
    this.tag = tag;
  }


  d(...args){
    console.log(`[${this.tag}]`, ...args);
    console.log('\n');
  }

  i(...args){
    console.info(`[${this.tag}]`, ...args);
  }

}

export default {
  Log,
  create(tag){
    return new Log(tag);
  }
}