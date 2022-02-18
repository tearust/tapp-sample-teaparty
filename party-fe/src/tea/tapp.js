import {_} from 'tearust_utils';
import utils from './utils';


const F = {
  setNickName(address, name){
    if(!address) throw 'invalid address.';
    const key = 'nickname__'+address;

    utils.cache.put(key, name);
  },
  getNickName(address){
    if(!address) throw 'invalid address.';
    const key = 'nickname__'+address;

    return utils.cache.get(key);
  }
};


window.tapp = F;
export default F;