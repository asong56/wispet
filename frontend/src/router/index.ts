import MainWindow from '@/view/main/index.vue';
import DictWindow from '@/view/dict/index.vue';
import SettingWindow from '@/view/setting/index.vue';
import BookmarksWindow from '@/view/bookmarks/index.vue';

import SettingDict from '@/view/setting/SettingDict.vue';
import SettingSoftware from '@/view/setting/SettingSoftware.vue';

export default [
  { path: '/', component: MainWindow },
  { path: '/dict', component: DictWindow },
  {
    path: '/setting',
    component: SettingWindow,
    children: [
      { path: '', redirect: '/setting/dict' },
      { path: 'dict', component: SettingDict },
      { path: 'software', component: SettingSoftware },
    ],
  },
  { path: '/bookmarks', component: BookmarksWindow },
];
