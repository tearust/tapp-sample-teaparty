import {uuid} from 'tearust_utils/index.cjs';

describe('test gluon utils', ()=>{

  test('it should be length 36 for uuid result', ()=>{
    const rs = uuid();

    expect(rs.length).toBe(36);
  })
});