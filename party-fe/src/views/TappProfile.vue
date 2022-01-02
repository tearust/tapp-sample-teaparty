<template>
<div class="tea-page">
  <h4>TApp profile</h4>

  <el-button type="primary" @click="updateHanlder">update</el-button>
</div>
</template>
<script>
import { mapGetters, mapState } from 'vuex';
import bbs from './bbs';
export default {
  data(){
    return {};
  },
  computed: {
    ...mapState(['user']),
    ...mapGetters([
      'layer1_account'
    ]),
  },
  methods: {
    async updateHanlder(){
      await bbs.updateTappProfile(this.layer1_account.address);
    }
  }
}
</script>