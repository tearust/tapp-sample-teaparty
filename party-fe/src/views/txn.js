import {_, axios, moment, uuid} from 'tearust_utils';
import utils from '../tea/utils';
import bbs from './bbs';

const F = {

  async txn_request(method, param){
    const _uuid = uuid();
    console.log("prepare for txn: ", method, _uuid);
    
    const _axios = bbs.getAxios();

    const txn_uuid = 'txn_'+_uuid;
    try{
      bbs.log("Send txn request...");
      console.log("Send txn request...");
      const step1_rs = await _axios.post('/tapp/'+method, {
        ...param,
        uuid: txn_uuid,
      });
      console.log("step_1 result: ", step1_rs);
    }catch(e){
      console.error("step_1 error: ", e);
      if(e === 'not_login'){
        throw e;
      }
    }

    bbs.log('Wait for query txn hash...');
    console.log('Wait for query txn hash...');
    await utils.sleep(5000);

    let step_2_rs = null;
    const step_2_loop = async ()=>{
      try{
        console.log('query result for '+txn_uuid+'...');
        step_2_rs = await _axios.post('/tapp/query_result', {
          uuid: txn_uuid,
        });

        step_2_rs = utils.parseJSON(step_2_rs);
      }catch(e){
        console.log("step2 error: ", e);
        // rs = e.message;
        step_2_rs = null;
        await utils.sleep(3000);
        await step_2_loop();
      }
  
    };
  
    bbs.log("Start to query txn result...");
    console.log("Start to query txn result...");
    await step_2_loop();

    console.log("step2 result: ", step_2_rs);

    bbs.log('Wait for query txn hash result...');
    console.log('Wait for query txn hash result...');
    utils.sleep(5000);

    const step_3_hash = step_2_rs.hash;
    const hash_uuid = "hash_"+_uuid;

    bbs.log("Start to send query txn hash request...");
    console.log('Start to send query txn hash request...');
    const step_3_rs = await _axios.post('/tapp/queryHashResult', {
      hash: step_3_hash,
      uuid: hash_uuid,
    });

    console.log("step3 result: ", step_3_rs);

    bbs.log('Wait for query txn hash result...');
    console.log('Wait for query txn hash result...');
    await utils.sleep(10000);

    let step_4_rs = null;
    let sn = 0;
    const step_4_loop = async ()=>{
      if(sn > 5) {
        step_4_rs = {
          'status': false,
          'error': 'request timeout',
        };
        return;
      }
      try{
        console.log('query hash result for '+hash_uuid+'...');
        step_4_rs = await _axios.post('/tapp/query_result', {
          uuid: hash_uuid,
        });
console.log(111, step_4_rs)
        step_4_rs = utils.parseJSON(step_4_rs);
        if(!step_4_rs.status) throw 'continue';
      }catch(e){
        console.log("step4 error: ", e);
        // rs = e.message;
        step_4_rs = null;
        sn++;
        await utils.sleep(3000);
        await step_4_loop();
      }
  
    };
  
    bbs.log("Start to query hash result...");
    console.log("Start to query hash result...");
    await step_4_loop();

    console.log("step4 result: ", step_4_rs);
    if(step_4_rs.error){
      throw step_4_rs.error;
    }

    const rs = step_4_rs.status;

    return rs;
  }

};


export default F;