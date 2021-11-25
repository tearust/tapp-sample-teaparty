import axios from 'axios';
import _ from 'lodash';
import utils from './tea/utils';

const SERVER_URL = utils.get_env('SERVER_URL');
const LAYER1_RPC = utils.get_env('LAYER1_RPC');
const BASE_URL = `${SERVER_URL}/`;

console.log("server_url => "+BASE_URL);

//set request base url
const _axios = axios.create({
  baseURL: BASE_URL,
});

// set request header 
_axios.interceptors.request.use((config)=>{
  
  return config;
});

// set request response
_axios.interceptors.response.use((res)=>{
  if(res.data){
    if(res.data.data){
      return Promise.resolve(res.data.data);
    }
    else{
      return Promise.reject(res.data);
    }
  }
}, (error)=>{
  return Promise.reject(error);
});

const help = {
  buildFilter(filter){
    if(!filter) return '';

    let str = '';
    _.each(filter, (val, key)=>{
      if(key === 'or'){
        str += key+': '+val+'\n';
      }
      else{
        str += key+': {'+val+'}\n';
      }
      
    });

    return `filter: {
      ${str}
    }`;
  }
};

const F = {
  SERVER_URL,

  async queryGraphQL(query, variables=null){
    const option = {
      query,
    };
    if(variables){
      option.variables = variables;
    }
    return await _axios.post('', option);
  },
  
  async getLog(filter){
    const query_filter = help.buildFilter(filter);
// console.log(11, query_filter)
    const query = `
query {
  logs (
    first: 100
    orderBy: AT_BLOCK_DESC
    ${filter ? query_filter : ''}
  ) {
    totalCount
    nodes {
      name
      type
      args
      atBlock
      from
      to
      auctionId
      cmlId
      price
      amount,
      tappId,
      json,
    }
  }
}
    `;
    const result = await F.queryGraphQL(query);
    return result.logs;
  },



  async layer1_rpc(method, params=[]){
    const data = {
      jsonrpc: '2.0',
      method,
      params,
      id: 9999
    };
    const rs = await axios.post(LAYER1_RPC, data, {
      headers: {
        'Content-Type': 'application/json',
      }
    });

    if(rs.data.error){
      throw rs.data.error;
    }

    if(rs.data.id === 9999){
      return rs.data.result;
    }

    return null;
  }
};


export default F;