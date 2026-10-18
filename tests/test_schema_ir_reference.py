"""Closed provider-neutral introspection reference: not yet a DB adapter."""
import hashlib,json,re,unittest
ALLOWED={'uuid','boolean','integer','text','decimal','timestamp'}
def canonical(tables):
    out=[];names=set()
    for t in tables:
        if set(t)!={'name','columns'} or not re.fullmatch('[a-z_][a-z0-9_]{0,62}',t['name']) or t['name'] in names:
            raise ValueError('bad table')
        names.add(t['name']);columns=[];seen=set()
        for c in t['columns']:
            if set(c)!={'name','kind','nullable'} or not re.fullmatch('[a-z_][a-z0-9_]{0,62}',c['name']) or c['name'] in seen or c['kind'] not in ALLOWED or type(c['nullable']) is not bool:
                raise ValueError('bad column')
            seen.add(c['name']);columns.append(c)
        if not columns:raise ValueError('no columns')
        out.append({'name':t['name'],'columns':sorted(columns,key=lambda x:x['name'])})
    wire=json.dumps({'version':1,'tables':sorted(out,key=lambda x:x['name'])},sort_keys=True,separators=(',',':'))
    return hashlib.sha256(wire.encode()).hexdigest()
class IntrospectionReferenceTest(unittest.TestCase):
    def setUp(self):
        self.t=[{'name':'users','columns':[{'name':'id','kind':'uuid','nullable':False},
          {'name':'active','kind':'boolean','nullable':False}]},
          {'name':'notes','columns':[{'name':'body','kind':'text','nullable':True}]}]
    def test_provider_order_is_not_identity(self):
        self.assertEqual(canonical(self.t),canonical(list(reversed(self.t))))
    def test_unsafe_unknown_fields_denied(self):
        with self.assertRaises(ValueError):canonical([{'name':'x','columns':[{'name':'raw','kind':'raw_sql','nullable':True}]}])
        with self.assertRaises(ValueError):canonical(self.t+self.t[:1])
if __name__=='__main__':unittest.main()
