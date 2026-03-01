import * as vscode from 'vscode';

const builtinModules = [
    {
        fullPath: 'core.io.Console',
        shortName: 'Console',
        members: [
            { name: 'writeLine', detail: 'fn writeLine(message: String)', documentation: 'Imprime mensagem com nova linha' },
            { name: 'write', detail: 'fn write(message: String)', documentation: 'Imprime mensagem sem nova linha' },
            { name: 'log', detail: 'fn log(message: String)', documentation: 'Log de informação' },
            { name: 'error', detail: 'fn error(message: String)', documentation: 'Log de erro' },
            { name: 'warn', detail: 'fn warn(message: String)', documentation: 'Log de aviso' },
            { name: 'info', detail: 'fn info(message: String)', documentation: 'Log de info' },
            { name: 'debug', detail: 'fn debug(message: String)', documentation: 'Log de debug' },
            { name: 'clear', detail: 'fn clear()', documentation: 'Limpa o console' },
            { name: 'readLine', detail: 'fn readLine() -> String', documentation: 'Lê linha do console' },
        ]
    },
    {
        fullPath: 'core.path.Path',
        shortName: 'Path',
        members: [
            { name: 'new', detail: 'fn new(path: &str) -> Path', documentation: 'Cria novo path' },
            { name: 'exists', detail: 'fn exists(&self) -> bool', documentation: 'Verifica se existe' },
            { name: 'isFile', detail: 'fn isFile(&self) -> bool', documentation: 'Verifica se é arquivo' },
            { name: 'isDir', detail: 'fn isDir(&self) -> bool', documentation: 'Verifica se é diretório' },
            { name: 'isAbsolute', detail: 'fn isAbsolute(&self) -> bool', documentation: 'Verifica se é absoluto' },
            { name: 'isRelative', detail: 'fn isRelative(&self) -> bool', documentation: 'Verifica se é relativo' },
            { name: 'extension', detail: 'fn extension(&self) -> Option<String>', documentation: 'Retorna extensão' },
            { name: 'fileName', detail: 'fn fileName(&self) -> Option<String>', documentation: 'Retorna nome do arquivo' },
            { name: 'parent', detail: 'fn parent(&self) -> Option<Path>', documentation: 'Retorna diretório pai' },
            { name: 'join', detail: 'fn join(&self, other: &str) -> Path', documentation: 'Junta paths' },
            { name: 'readDir', detail: 'fn readDir(&self) -> Vec<DirEntry>', documentation: 'Lista diretório' },
            { name: 'toString', detail: 'fn toString(&self) -> String', documentation: 'Converte para string' },
            { name: 'toAbsolute', detail: 'fn toAbsolute(&self) -> Path', documentation: 'Converte para absoluto' },
            { name: 'normalize', detail: 'fn normalize(&self) -> Path', documentation: 'Normaliza path' },
            { name: 'stripPrefix', detail: 'fn stripPrefix(&self, base: &Path) -> Path', documentation: 'Remove prefixo' },
            { name: 'components', detail: 'fn components(&self) -> Vec<PathBuf>', documentation: 'Componentes do path' },
            { name: 'startsWith', detail: 'fn startsWith(&self, base: &Path) -> bool', documentation: 'Verifica início' },
            { name: 'endsWith', detail: 'fn endsWith(&self, component: &str) -> bool', documentation: 'Verifica final' },
            { name: 'metadata', detail: 'fn metadata(&self) -> Metadata', documentation: 'Metadados do arquivo' },
            { name: 'symlinkMetadata', detail: 'fn symlinkMetadata(&self) -> Metadata', documentation: 'Metadados do symlink' },
            { name: 'copy', detail: 'fn copy(&self, to: &Path) -> Result<(), Error>', documentation: 'Copia arquivo' },
            { name: 'createDir', detail: 'fn createDir(&self, recursive: bool) -> Result<(), Error>', documentation: 'Cria diretório' },
            { name: 'createDirAll', detail: 'fn createDirAll(&self) -> Result<(), Error>', documentation: 'Cria diretórios recursivamente' },
            { name: 'removeDir', detail: 'fn removeDir(&self) -> Result<(), Error>', documentation: 'Remove diretório' },
            { name: 'removeDirAll', detail: 'fn removeDirAll(&self) -> Result<(), Error>', documentation: 'Remove diretório recursivamente' },
            { name: 'rename', detail: 'fn rename(&self, new: &Path) -> Result<(), Error>', documentation: 'Renomeia arquivo' },
            { name: 'setPermissions', detail: 'fn setPermissions(&self, perm: Permissions) -> Result<(), Error>', documentation: 'Define permissões' },
        ]
    },
    {
        fullPath: 'core.object.Object',
        shortName: 'Object',
        members: [
            { name: 'new', detail: 'fn new() -> Object', documentation: 'Cria novo objeto vazio' },
            { name: 'from', detail: 'fn from<T>(value: T) -> Object', documentation: 'Cria objeto de valor' },
            { name: 'get', detail: 'fn get(&self, key: &str) -> Option<Value>', documentation: 'Obtém valor por chave' },
            { name: 'set', detail: 'fn set(&mut self, key: String, value: Value)', documentation: 'Define valor por chave' },
            { name: 'has', detail: 'fn has(&self, key: &str) -> bool', documentation: 'Verifica se chave existe' },
            { name: 'delete', detail: 'fn delete(&mut self, key: &str) -> Option<Value>', documentation: 'Remove chave' },
            { name: 'keys', detail: 'fn keys(&self) -> Vec<String>', documentation: 'Retorna todas as chaves' },
            { name: 'values', detail: 'fn values(&self) -> Vec<Value>', documentation: 'Retorna todos os valores' },
            { name: 'entries', detail: 'fn entries(&self) -> Vec<(String, Value)>', documentation: 'Retorna pares' },
            { name: 'merge', detail: 'fn merge(&mut self, other: Object)', documentation: 'Mescla objetos' },
            { name: 'clone', detail: 'fn clone(&self) -> Object', documentation: 'Clona objeto' },
            { name: 'toJson', detail: 'fn toJson(&self) -> String', documentation: 'Converte para JSON' },
            { name: 'fromJson', detail: 'fn fromJson(json: &str) -> Object', documentation: 'Cria de JSON' },
            { name: 'isEmpty', detail: 'fn isEmpty(&self) -> bool', documentation: 'Verifica se vazio' },
            { name: 'len', detail: 'fn len(&self) -> usize', documentation: 'Retorna tamanho' },
            { name: 'forEach', detail: 'fn forEach(&self, f: fn(&str, &Value))', documentation: 'Itera sobre entradas' },
        ]
    },
    {
        fullPath: 'core.type.Type',
        shortName: 'Type',
        members: [
            { name: 'of', detail: 'fn of<T>() -> Type', documentation: 'Obtém tipo de T' },
            { name: 'name', detail: 'fn name(&self) -> String', documentation: 'Retorna nome do tipo' },
            { name: 'kind', detail: 'fn kind(&self) -> TypeKind', documentation: 'Retorna kind do tipo' },
            { name: 'size', detail: 'fn size(&self) -> usize', documentation: 'Retorna tamanho em bytes' },
            { name: 'align', detail: 'fn align(&self) -> usize', documentation: 'Retorna alinhamento' },
            { name: 'isPrimitive', detail: 'fn isPrimitive(&self) -> bool', documentation: 'Verifica se é primitivo' },
            { name: 'isReference', detail: 'fn isReference(&self) -> bool', documentation: 'Verifica se é referência' },
            { name: 'isPointer', detail: 'fn isPointer(&self) -> bool', documentation: 'Verifica se é ponteiro' },
            { name: 'isArray', detail: 'fn isArray(&self) -> bool', documentation: 'Verifica se é array' },
            { name: 'isSlice', detail: 'fn isSlice(&self) -> bool', documentation: 'Verifica se é slice' },
            { name: 'isStruct', detail: 'fn isStruct(&self) -> bool', documentation: 'Verifica se é struct' },
            { name: 'isEnum', detail: 'fn isEnum(&self) -> bool', documentation: 'Verifica se é enum' },
            { name: 'isTrait', detail: 'fn isTrait(&self) -> bool', documentation: 'Verifica se é trait' },
            { name: 'fields', detail: 'fn fields(&self) -> Vec<Field>', documentation: 'Retorna campos' },
            { name: 'methods', detail: 'fn methods(&self) -> Vec<Method>', documentation: 'Retorna métodos' },
            { name: 'implements', detail: 'fn implements(&self, trait: &Type) -> bool', documentation: 'Verifica implementação' },
            { name: 'cast', detail: 'fn cast<T>(&self) -> Option<T>', documentation: 'Cast para tipo' },
        ]
    },
    {
        fullPath: 'core.description.Description',
        shortName: 'Description',
        members: [
            { name: 'new', detail: 'fn new(title: String) -> Description', documentation: 'Cria nova descrição' },
            { name: 'setTitle', detail: 'fn setTitle(&mut self, title: String)', documentation: 'Define título' },
            { name: 'setText', detail: 'fn setText(&mut self, text: String)', documentation: 'Define texto' },
            { name: 'addField', detail: 'fn addField(&mut self, key: String, value: String)', documentation: 'Adiciona campo' },
            { name: 'addImage', detail: 'fn addImage(&mut self, url: String)', documentation: 'Adiciona imagem' },
            { name: 'addButton', detail: 'fn addButton(&mut self, label: String, action: String)', documentation: 'Adiciona botão' },
            { name: 'addLink', detail: 'fn addLink(&mut self, label: String, url: String)', documentation: 'Adiciona link' },
            { name: 'setFooter', detail: 'fn setFooter(&mut self, text: String)', documentation: 'Define rodapé' },
            { name: 'setColor', detail: 'fn setColor(&mut self, color: String)', documentation: 'Define cor' },
            { name: 'setTimestamp', detail: 'fn setTimestamp(&mut self, ts: i64)', documentation: 'Define timestamp' },
            { name: 'toJson', detail: 'fn toJson(&self) -> String', documentation: 'Converte para JSON' },
            { name: 'build', detail: 'fn build(self) -> String', documentation: 'Constrói descrição' },
        ]
    },
    {
        fullPath: 'core.enum.Enum',
        shortName: 'Enum',
        members: [
            { name: 'fromStr', detail: 'fn fromStr<T: Enum>(s: &str) -> Option<T>', documentation: 'Parseia de string' },
            { name: 'variants', detail: 'fn variants<T: Enum>() -> Vec<T>', documentation: 'Lista variantes' },
            { name: 'name', detail: 'fn name(&self) -> String', documentation: 'Retorna nome da variante' },
            { name: 'index', detail: 'fn index(&self) -> usize', documentation: 'Retorna índice' },
            { name: 'isVariant', detail: 'fn isVariant<T: Enum>(&self, variant: &T) -> bool', documentation: 'Verifica variante' },
            { name: 'tryFromIndex', detail: 'fn tryFromIndex<T: Enum>(index: usize) -> Option<T>', documentation: 'De índice' },
            { name: 'toStr', detail: 'fn toStr(&self) -> &str', documentation: 'Converte para string' },
        ]
    },
    {
        fullPath: 'core.reflect.Reflect',
        shortName: 'Reflect',
        members: [
            { name: 'typeOf', detail: 'fn typeOf<T>() -> Type', documentation: 'Obtém tipo de valor' },
            { name: 'typeName', detail: 'fn typeName<T>() -> String', documentation: 'Nome do tipo' },
            { name: 'isType', detail: 'fn isType<T, U>() -> bool', documentation: 'Verifica tipo' },
            { name: 'getField', detail: 'fn getField<T>(obj: &T, name: &str) -> Option<Value>', documentation: 'Obtém campo' },
            { name: 'setField', detail: 'fn setField<T>(obj: &mut T, name: &str, value: Value) -> bool', documentation: 'Define campo' },
            { name: 'fields', detail: 'fn fields<T>() -> Vec<FieldInfo>', documentation: 'Lista campos' },
            { name: 'methods', detail: 'fn methods<T>() -> Vec<MethodInfo>', documentation: 'Lista métodos' },
            { name: 'callMethod', detail: 'fn callMethod<T>(obj: &T, name: &str, args: Vec<Value>) -> Result<Value, Error>', documentation: 'Chama método' },
            { name: 'createInstance', detail: 'fn createInstance<T>(args: Vec<Value>) -> Result<T, Error>', documentation: 'Cria instância' },
            { name: 'serialize', detail: 'fn serialize<T>(obj: &T) -> Result<String, Error>', documentation: 'Serializa' },
            { name: 'deserialize', detail: 'fn deserialize<T>(data: &str) -> Result<T, Error>', documentation: 'Deserializa' },
        ]
    },
    {
        fullPath: 'core.validation.Validator',
        shortName: 'Validator',
        members: [
            { name: 'new', detail: 'fn new() -> Validator', documentation: 'Cria novo validador' },
            { name: 'validate', detail: 'fn validate<T>(&self, value: &T) -> Result<(), ValidationError>', documentation: 'Valida valor' },
            { name: 'addRule', detail: 'fn addRule<T, F>(&mut self, rule: F) where F: Rule<T>', documentation: 'Adiciona regra' },
            { name: 'required', detail: 'fn required<T>(&mut self) -> &mut Self', documentation: 'Campo obrigatório' },
            { name: 'min', detail: 'fn min<T: PartialOrd>(&mut self, min: T) -> &mut Self', documentation: 'Valor mínimo' },
            { name: 'max', detail: 'fn max<T: PartialOrd>(&mut self, max: T) -> &mut Self', documentation: 'Valor máximo' },
            { name: 'length', detail: 'fn length(&mut self, min: usize, max: usize) -> &mut Self', documentation: 'Tamanho' },
            { name: 'email', detail: 'fn email(&mut self) -> &mut Self', documentation: 'Valida email' },
            { name: 'url', detail: 'fn url(&mut self) -> &mut Self', documentation: 'Valida URL' },
            { name: 'pattern', detail: 'fn pattern(&mut self, regex: &str) -> &mut Self', documentation: 'Valida padrão regex' },
            { name: 'custom', detail: 'fn custom<F>(&mut self, f: F) -> &mut Self where F: Fn(&Value) -> bool', documentation: 'Validação customizada' },
        ]
    },
    {
        fullPath: 'core.clone.Clone',
        shortName: 'Clone',
        members: [
            { name: 'clone', detail: 'fn clone<T: Clone>(value: &T) -> T', documentation: 'Clona valor' },
            { name: 'cloneFrom', detail: 'fn cloneFrom<T: Clone>(target: &mut T, source: &T)', documentation: 'Clona para destino' },
            { name: 'deepClone', detail: 'fn deepClone<T: DeepClone>(value: &T) -> T', documentation: 'Clonagem profunda' },
            { name: 'isClone', detail: 'fn isClone<T, U>() -> bool', documentation: 'Verifica se implementa Clone' },
        ]
    },
    {
        fullPath: 'core.convert.Convert',
        shortName: 'Convert',
        members: [
            { name: 'into', detail: 'fn into<T, U>(value: T) -> U where T: Into<U>', documentation: 'Converte com Into' },
            { name: 'from', detail: 'fn from<T, U>(value: T) -> U where U: From<T>', documentation: 'Converte com From' },
            { name: 'as', detail: 'fn as<T, U>(value: T) -> U where T: AsRef<U>', documentation: 'Converte com AsRef' },
            { name: 'tryInto', detail: 'fn tryInto<T, U>(value: T) -> Result<U, Error> where T: TryInto<U>', documentation: 'Tenta converter' },
            { name: 'fromStr', detail: 'fn fromStr<T: FromStr>(s: &str) -> Option<T>', documentation: 'Parseia de string' },
            { name: 'toString', detail: 'fn toString<T: ToString>(value: &T) -> String', documentation: 'Converte para string' },
            { name: 'toOwned', detail: 'fn toOwned<T: ToOwned>(value: &T) -> T::Owned', documentation: 'Converte para owned' },
        ]
    },
    {
        fullPath: 'core.compare.Compare',
        shortName: 'Compare',
        members: [
            { name: 'eq', detail: 'fn eq<T: PartialEq>(a: &T, b: &T) -> bool', documentation: 'Verifica igualdade' },
            { name: 'ne', detail: 'fn ne<T: PartialEq>(a: &T, b: &T) -> bool', documentation: 'Verifica desigualdade' },
            { name: 'lt', detail: 'fn lt<T: PartialOrd>(a: &T, b: &T) -> bool', documentation: 'Menor que' },
            { name: 'le', detail: 'fn le<T: PartialOrd>(a: &T, b: &T) -> bool', documentation: 'Menor ou igual' },
            { name: 'gt', detail: 'fn gt<T: PartialOrd>(a: &T, b: &T) -> bool', documentation: 'Maior que' },
            { name: 'ge', detail: 'fn ge<T: PartialOrd>(a: &T, b: &T) -> bool', documentation: 'Maior ou igual' },
            { name: 'compare', detail: 'fn compare<T: PartialOrd>(a: &T, b: &T) -> Ordering', documentation: 'Compara valores' },
            { name: 'max', detail: 'fn max<T: PartialOrd>(a: T, b: T) -> T', documentation: 'Retorna máximo' },
            { name: 'min', detail: 'fn min<T: PartialOrd>(a: T, b: T) -> T', documentation: 'Retorna mínimo' },
            { name: 'clamp', detail: 'fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T', documentation: 'Limita valor' },
        ]
    },
    {
        fullPath: 'core.format.Format',
        shortName: 'Format',
        members: [
            { name: 'format', detail: 'fn format(fmt: &str, args: ...) -> String', documentation: 'Formata string' },
            { name: 'formatArgs', detail: 'fn formatArgs(fmt: &str, args: Args) -> String', documentation: 'Formata com Args' },
            { name: 'formatList', detail: 'fn formatList(fmt: &str, list: Vec<Value>) -> String', documentation: 'Formata lista' },
            { name: 'formatStruct', detail: 'fn formatStruct(fmt: &str, obj: &Object) -> String', documentation: 'Formata struct' },
            { name: 'formatNumber', detail: 'fn formatNumber(n: f64, fmt: &str) -> String', documentation: 'Formata número' },
            { name: 'formatDateTime', detail: 'fn formatDateTime(dt: &DateTime, fmt: &str) -> String', documentation: 'Formata data/hora' },
            { name: 'formatDuration', detail: 'fn formatDuration(d: Duration, fmt: &str) -> String', documentation: 'Formata duração' },
            { name: 'formatJson', detail: 'fn formatJson(value: &Value, pretty: bool) -> String', documentation: 'Formata JSON' },
            { name: 'formatTable', detail: 'fn formatTable(headers: Vec<&str>, rows: Vec<Vec<&str>>) -> String', documentation: 'Formata tabela' },
        ]
    },
    {
        fullPath: 'core.macro.Macro',
        shortName: 'Macro',
        members: [
            { name: 'define', detail: 'fn define(name: &str, body: &str)', documentation: 'Define macro' },
            { name: 'expand', detail: 'fn expand(macro: &str, args: Vec<&str>) -> String', documentation: 'Expande macro' },
            { name: 'call', detail: 'fn call(name: &str, args: Vec<&str>) -> String', documentation: 'Chama macro' },
            { name: 'list', detail: 'fn list() -> Vec<String>', documentation: 'Lista macros' },
            { name: 'remove', detail: 'fn remove(name: &str)', documentation: 'Remove macro' },
            { name: 'exists', detail: 'fn exists(name: &str) -> bool', documentation: 'Verifica se macro existe' },
        ]
    },
    {
        fullPath: 'core.test.Test',
        shortName: 'Test',
        members: [
            { name: 'describe', detail: 'fn describe(name: &str, f: fn())', documentation: 'Descreve suite de testes' },
            { name: 'it', detail: 'fn it(name: &str, f: fn())', documentation: 'Define teste' },
            { name: 'before', detail: 'fn before(f: fn())', documentation: 'Executa antes de cada teste' },
            { name: 'after', detail: 'fn after(f: fn())', documentation: 'Executa depois de cada teste' },
            { name: 'beforeAll', detail: 'fn beforeAll(f: fn())', documentation: 'Executa antes de todos testes' },
            { name: 'afterAll', detail: 'fn afterAll(f: fn())', documentation: 'Executa depois de todos testes' },
            { name: 'skip', detail: 'fn skip(reason: &str)', documentation: 'Pula teste' },
            { name: 'only', detail: 'fn only()', documentation: 'Executa apenas este teste' },
            { name: 'todo', detail: 'fn todo(msg: &str)', documentation: 'Marca como pendente' },
            { name: 'assert', detail: 'fn assert(cond: bool, msg: &str)', documentation: 'Assert customizado' },
            { name: 'assertEq', detail: 'fn assertEq<T: PartialEq>(a: T, b: T, msg: &str)', documentation: 'Assert igualdade' },
            { name: 'assertNe', detail: 'fn assertNe<T: PartialEq>(a: T, b: T, msg: &str)', documentation: 'Assert desigualdade' },
            { name: 'assertTrue', detail: 'fn assertTrue(cond: bool, msg: &str)', documentation: 'Assert verdadeiro' },
            { name: 'assertFalse', detail: 'fn assertFalse(cond: bool, msg: &str)', documentation: 'Assert falso' },
            { name: 'assertNull', detail: 'fn assertNull<T>(value: &T, msg: &str)', documentation: 'Assert nulo' },
            { name: 'assertNotNull', detail: 'fn assertNotNull<T>(value: &T, msg: &str)', documentation: 'Assert não nulo' },
            { name: 'assertThrows', detail: 'fn assertThrows<T>(f: fn() -> T, msg: &str)', documentation: 'Assert lança erro' },
            { name: 'run', detail: 'fn run() -> TestResult', documentation: 'Executa todos os testes' },
        ]
    },
    {
        fullPath: 'core.net.Http',
        shortName: 'Http',
        members: [
            { name: 'serve', detail: 'fn serve(address: String, app: Fn)', documentation: 'Inicia servidor HTTP' },
            { name: 'get', detail: 'fn get(url: String) -> Response', documentation: 'Requisição GET' },
            { name: 'post', detail: 'fn post(url: String, body: String) -> Response', documentation: 'Requisição POST' },
            { name: 'put', detail: 'fn put(url: String, body: String) -> Response', documentation: 'Requisição PUT' },
            { name: 'delete', detail: 'fn delete(url: String) -> Response', documentation: 'Requisição DELETE' },
            { name: 'patch', detail: 'fn patch(url: String, body: String) -> Response', documentation: 'Requisição PATCH' },
            { name: 'head', detail: 'fn head(url: String) -> Response', documentation: 'Requisição HEAD' },
            { name: 'options', detail: 'fn options(url: String) -> Response', documentation: 'Requisição OPTIONS' },
        ]
    },
    {
        fullPath: 'core.json.Json',
        shortName: 'Json',
        members: [
            { name: 'parse', detail: 'fn parse<T>(json: String) -> T', documentation: 'Parseia string JSON para objeto' },
            { name: 'stringify', detail: 'fn stringify(value: any) -> String', documentation: 'Converte objeto para string JSON' },
            { name: 'get', detail: 'fn get(json: JsonValue, key: String) -> JsonValue', documentation: 'Obtém valor de chave' },
            { name: 'set', detail: 'fn set(json: &JsonValue, key: String, value: JsonValue)', documentation: 'Define valor de chave' },
            { name: 'parseArray', detail: 'fn parseArray<T>(json: String) -> Vec<T>', documentation: 'Parseia JSON array' },
            { name: 'fromObject', detail: 'fn fromObject(obj: &Object) -> JsonValue', documentation: 'Cria JsonValue de objeto' },
        ]
    },
    {
        fullPath: 'core.env.Env',
        shortName: 'Env',
        members: [
            { name: 'get', detail: 'fn get(key: String) -> Option<String>', documentation: 'Obtém variável de ambiente' },
            { name: 'set', detail: 'fn set(key: String, value: String)', documentation: 'Define variável de ambiente' },
            { name: 'vars', detail: 'fn vars() -> HashMap<String, String>', documentation: 'Lista todas variáveis' },
            { name: 'remove', detail: 'fn remove(key: String)', documentation: 'Remove variável de ambiente' },
            { name: 'exists', detail: 'fn exists(key: String) -> bool', documentation: 'Verifica se variável existe' },
        ]
    },
    {
        fullPath: 'core.collections.List',
        shortName: 'List',
        members: [
            { name: 'new', detail: 'fn new<T>() -> List<T>', documentation: 'Cria nova lista vazia' },
            { name: 'push', detail: 'fn push(&self, item: T)', documentation: 'Adiciona item à lista' },
            { name: 'pop', detail: 'fn pop(&self) -> Option<T>', documentation: 'Remove último item' },
            { name: 'get', detail: 'fn get(&self, index: usize) -> Option<&T>', documentation: 'Obtém item por índice' },
            { name: 'set', detail: 'fn set(&mut self, index: usize, value: T)', documentation: 'Define item por índice' },
            { name: 'len', detail: 'fn len(&self) -> usize', documentation: 'Retorna tamanho da lista' },
            { name: 'isEmpty', detail: 'fn isEmpty(&self) -> bool', documentation: 'Verifica se lista vazia' },
            { name: 'clear', detail: 'fn clear(&mut self)', documentation: 'Limpa todos os itens' },
            { name: 'contains', detail: 'fn contains(&self, item: &T) -> bool', documentation: 'Verifica se contém item' },
            { name: 'indexOf', detail: 'fn indexOf(&self, item: &T) -> Option<usize>', documentation: 'Retorna índice do item' },
            { name: 'remove', detail: 'fn remove(&mut self, index: usize) -> Option<T>', documentation: 'Remove item por índice' },
            { name: 'map', detail: 'fn map<U>(&self, f: fn(T) -> U) -> List<U>', documentation: 'Aplica função a todos' },
            { name: 'filter', detail: 'fn filter(&self, f: fn(&T) -> bool) -> List<T>', documentation: 'Filtra elementos' },
            { name: 'reduce', detail: 'fn reduce<U>(&self, init: U, f: fn(U, &T) -> U) -> U', documentation: 'Reduce elementos' },
            { name: 'forEach', detail: 'fn forEach(&self, f: fn(&T))', documentation: 'Itera sobre elementos' },
            { name: 'toArray', detail: 'fn toArray(&self) -> Vec<T>', documentation: 'Converte para array' },
            { name: 'fromArray', detail: 'fn fromArray(arr: Vec<T>) -> List<T>', documentation: 'Cria de array' },
            { name: 'sort', detail: 'fn sort(&mut self)', documentation: 'Ordena lista' },
            { name: 'reverse', detail: 'fn reverse(&mut self)', documentation: 'Inverte lista' },
            { name: 'slice', detail: 'fn slice(&self, start: usize, end: usize) -> List<T>', documentation: 'Retorna fatia' },
        ]
    },
    {
        fullPath: 'core.collections.Map',
        shortName: 'Map',
        members: [
            { name: 'new', detail: 'fn new<K, V>() -> Map<K, V>', documentation: 'Cria novo mapa vazio' },
            { name: 'set', detail: 'fn set(&mut self, key: K, value: V)', documentation: 'Define valor para chave' },
            { name: 'get', detail: 'fn get(&self, key: &K) -> Option<&V>', documentation: 'Obtém valor por chave' },
            { name: 'has', detail: 'fn has(&self, key: &K) -> bool', documentation: 'Verifica se chave existe' },
            { name: 'remove', detail: 'fn remove(&mut self, key: &K) -> Option<V>', documentation: 'Remove chave' },
            { name: 'len', detail: 'fn len(&self) -> usize', documentation: 'Retorna tamanho' },
            { name: 'isEmpty', detail: 'fn isEmpty(&self) -> bool', documentation: 'Verifica se vazio' },
            { name: 'clear', detail: 'fn clear(&mut self)', documentation: 'Limpa mapa' },
            { name: 'keys', detail: 'fn keys(&self) -> Vec<K>', documentation: 'Retorna todas as chaves' },
            { name: 'values', detail: 'fn values(&self) -> Vec<V>', documentation: 'Retorna todos os valores' },
            { name: 'entries', detail: 'fn entries(&self) -> Vec<(K, V)>', documentation: 'Retorna pares chave-valor' },
            { name: 'forEach', detail: 'fn forEach(&self, f: fn(&K, &V))', documentation: 'Itera sobre entradas' },
        ]
    },
    {
        fullPath: 'core.collections.Set',
        shortName: 'Set',
        members: [
            { name: 'new', detail: 'fn new<T>() -> Set<T>', documentation: 'Cria novo conjunto vazio' },
            { name: 'add', detail: 'fn add(&mut self, value: T)', documentation: 'Adiciona valor ao conjunto' },
            { name: 'has', detail: 'fn has(&self, value: &T) -> bool', documentation: 'Verifica se contém valor' },
            { name: 'remove', detail: 'fn remove(&mut self, value: &T) -> bool', documentation: 'Remove valor' },
            { name: 'len', detail: 'fn len(&self) -> usize', documentation: 'Retorna tamanho' },
            { name: 'isEmpty', detail: 'fn isEmpty(&self) -> bool', documentation: 'Verifica se vazio' },
            { name: 'clear', detail: 'fn clear(&mut self)', documentation: 'Limpa conjunto' },
            { name: 'union', detail: 'fn union(&self, other: &Set<T>) -> Set<T>', documentation: 'União de conjuntos' },
            { name: 'intersection', detail: 'fn intersection(&self, other: &Set<T>) -> Set<T>', documentation: 'Interseção' },
            { name: 'difference', detail: 'fn difference(&self, other: &Set<T>) -> Set<T>', documentation: 'Diferença' },
        ]
    },
    {
        fullPath: 'core.collections.Vec',
        shortName: 'Vec',
        members: [
            { name: 'new', detail: 'fn new<T>() -> Vec<T>', documentation: 'Cria novo vetor vazio' },
            { name: 'withCapacity', detail: 'fn withCapacity<T>(cap: usize) -> Vec<T>', documentation: 'Cria com capacidade' },
            { name: 'push', detail: 'fn push(&mut self, value: T)', documentation: 'Adiciona elemento' },
            { name: 'pop', detail: 'fn pop(&mut self) -> Option<T>', documentation: 'Remove último elemento' },
            { name: 'get', detail: 'fn get(&self, index: usize) -> Option<&T>', documentation: 'Obtém elemento' },
            { name: 'len', detail: 'fn len(&self) -> usize', documentation: 'Retorna tamanho' },
            { name: 'capacity', detail: 'fn capacity(&self) -> usize', documentation: 'Retorna capacidade' },
            { name: 'reserve', detail: 'fn reserve(&mut self, additional: usize)', documentation: 'Reserva capacidade' },
            { name: 'shrinkToFit', detail: 'fn shrinkToFit(&mut self)', documentation: 'Reduz capacidade' },
            { name: 'clear', detail: 'fn clear(&mut self)', documentation: 'Limpa vetor' },
            { name: 'toList', detail: 'fn toList(&self) -> List<T>', documentation: 'Converte para lista' },
        ]
    },
    {
        fullPath: 'core.string.String',
        shortName: 'String',
        members: [
            { name: 'from', detail: 'fn from(s: &str) -> String', documentation: 'Cria String de &str' },
            { name: 'fromChars', detail: 'fn fromChars(chars: Vec<char>) -> String', documentation: 'Cria de caracteres' },
            { name: 'len', detail: 'fn len(&self) -> usize', documentation: 'Retorna tamanho' },
            { name: 'isEmpty', detail: 'fn isEmpty(&self) -> bool', documentation: 'Verifica se vazio' },
            { name: 'chars', detail: 'fn chars(&self) -> Vec<char>', documentation: 'Retorna caracteres' },
            { name: 'bytes', detail: 'fn bytes(&self) -> Vec<u8>', documentation: 'Retorna bytes' },
            { name: 'contains', detail: 'fn contains(&self, s: &str) -> bool', documentation: 'Verifica substring' },
            { name: 'startsWith', detail: 'fn startsWith(&self, s: &str) -> bool', documentation: 'Verifica prefixo' },
            { name: 'endsWith', detail: 'fn endsWith(&self, s: &str) -> bool', documentation: 'Verifica sufixo' },
            { name: 'find', detail: 'fn find(&self, s: &str) -> Option<usize>', documentation: 'Encontra substring' },
            { name: 'replace', detail: 'fn replace(&self, from: &str, to: &str) -> String', documentation: 'Substitui substring' },
            { name: 'split', detail: 'fn split(&self, sep: &str) -> Vec<String>', documentation: 'Divide string' },
            { name: 'trim', detail: 'fn trim(&self) -> String', documentation: 'Remove espaços' },
            { name: 'toUpperCase', detail: 'fn toUpperCase(&self) -> String', documentation: 'Maiúsculas' },
            { name: 'toLowerCase', detail: 'fn toLowerCase(&self) -> String', documentation: 'Minúsculas' },
            { name: 'toString', detail: 'fn toString(&self) -> String', documentation: 'Converte para string' },
            { name: 'asStr', detail: 'fn asStr(&self) -> &str', documentation: 'Retorna &str' },
            { name: 'parse', detail: 'fn parse<T: FromStr>(s: &str) -> Option<T>', documentation: 'Parseia para tipo' },
            { name: 'format', detail: 'fn format(fmt: &str, args: ...) -> String', documentation: 'Formata string' },
            { name: 'join', detail: 'fn join(parts: &[String], sep: &str) -> String', documentation: 'Junta partes' },
            { name: 'repeat', detail: 'fn repeat(&self, n: usize) -> String', documentation: 'Repete string' },
            { name: 'isNumeric', detail: 'fn isNumeric(&self) -> bool', documentation: 'Verifica se numérico' },
            { name: 'isAlpha', detail: 'fn isAlpha(&self) -> bool', documentation: 'Verifica se alfabético' },
            { name: 'isAlphanumeric', detail: 'fn isAlphanumeric(&self) -> bool', documentation: 'Verifica se alfanumérico' },
        ]
    },
    {
        fullPath: 'core.time.DateTime',
        shortName: 'DateTime',
        members: [
            { name: 'now', detail: 'fn now() -> DateTime', documentation: 'Data/hora atual' },
            { name: 'fromTimestamp', detail: 'fn fromTimestamp(ts: i64) -> DateTime', documentation: 'De timestamp' },
            { name: 'fromStr', detail: 'fn fromStr(s: &str, fmt: &str) -> Option<DateTime>', documentation: 'De string' },
            { name: 'timestamp', detail: 'fn timestamp(&self) -> i64', documentation: 'Retorna timestamp' },
            { name: 'format', detail: 'fn format(&self, fmt: &str) -> String', documentation: 'Formata data' },
            { name: 'year', detail: 'fn year(&self) -> i32', documentation: 'Retorna ano' },
            { name: 'month', detail: 'fn month(&self) -> u32', documentation: 'Retorna mês' },
            { name: 'day', detail: 'fn day(&self) -> u32', documentation: 'Retorna dia' },
            { name: 'hour', detail: 'fn hour(&self) -> u32', documentation: 'Retorna hora' },
            { name: 'minute', detail: 'fn minute(&self) -> u32', documentation: 'Retorna minuto' },
            { name: 'second', detail: 'fn second(&self) -> u32', documentation: 'Retorna segundo' },
        ]
    },
    {
        fullPath: 'core.net.Grpc',
        shortName: 'Grpc',
        members: [
            { name: 'createServer', detail: 'fn createServer(config: GrpcConfig) -> GrpcServer', documentation: 'Cria servidor gRPC' },
            { name: 'createClient', detail: 'fn createClient(config: GrpcConfig) -> GrpcClient', documentation: 'Cria cliente gRPC' },
            { name: 'registerService', detail: 'fn registerService(&mut self, service: &Service)', documentation: 'Registra serviço' },
        ]
    },
    {
        fullPath: 'core.wasm.Env',
        shortName: 'Wasm',
        members: [
            { name: 'memory', detail: 'var memory: WebAssembly.Memory', documentation: 'Memória WASM' },
            { name: 'alloc', detail: 'fn alloc(size: usize) -> pointer', documentation: 'Aloca memória' },
            { name: 'dealloc', detail: 'fn dealloc(ptr: pointer)', documentation: 'Desaloca memória' },
            { name: 'readString', detail: 'fn readString(ptr: pointer) -> String', documentation: 'Lê string de memória' },
            { name: 'writeString', detail: 'fn writeString(s: &str) -> pointer', documentation: 'Escreve string em memória' },
        ]
    },
    {
        fullPath: 'core.option.Option',
        shortName: 'Option',
        members: [
            { name: 'some', detail: 'fn some<T>(value: T) -> Option<T>', documentation: 'Cria Option::Some' },
            { name: 'none', detail: 'fn none<T>() -> Option<T>', documentation: 'Cria Option::None' },
            { name: 'isSome', detail: 'fn isSome(&self) -> bool', documentation: 'Verifica se é Some' },
            { name: 'isNone', detail: 'fn isNone(&self) -> bool', documentation: 'Verifica se é None' },
            { name: 'unwrap', detail: 'fn unwrap(self) -> T', documentation: 'Desembrulha valor' },
            { name: 'unwrapOr', detail: 'fn unwrapOr(self, default: T) -> T', documentation: 'Desembrulha ou padrão' },
            { name: 'unwrapOrElse', detail: 'fn unwrapOrElse(self, f: fn() -> T) -> T', documentation: 'Desembrulha ou calcula' },
            { name: 'map', detail: 'fn map<U>(self, f: fn(T) -> U) -> Option<U>', documentation: 'Mapeia valor' },
            { name: 'mapOr', detail: 'fn mapOr<U>(self, default: U, f: fn(T) -> U) -> U', documentation: 'Mapeia ou padrão' },
            { name: 'andThen', detail: 'fn andThen<U>(self, f: fn(T) -> Option<U>) -> Option<U>', documentation: 'Encadeia option' },
            { name: 'or', detail: 'fn or(self, other: Option<T>) -> Option<T>', documentation: 'Alternativa' },
            { name: 'orElse', detail: 'fn orElse(self, f: fn() -> Option<T>) -> Option<T>', documentation: 'Alternativa calculada' },
            { name: 'contains', detail: 'fn contains(&self, value: &T) -> bool', documentation: 'Verifica valor' },
        ]
    },
    {
        fullPath: 'core.result.Result',
        shortName: 'Result',
        members: [
            { name: 'ok', detail: 'fn ok<T, E>(value: T) -> Result<T, E>', documentation: 'Cria Result::Ok' },
            { name: 'err', detail: 'fn err<T, E>(error: E) -> Result<T, E>', documentation: 'Cria Result::Err' },
            { name: 'isOk', detail: 'fn isOk(&self) -> bool', documentation: 'Verifica se é Ok' },
            { name: 'isErr', detail: 'fn isErr(&self) -> bool', documentation: 'Verifica se é Err' },
            { name: 'unwrap', detail: 'fn unwrap(self) -> T', documentation: 'Desembrulha sucesso' },
            { name: 'unwrapErr', detail: 'fn unwrapErr(self) -> E', documentation: 'Desembrulha erro' },
            { name: 'unwrapOr', detail: 'fn unwrapOr(self, default: T) -> T', documentation: 'Desembrulha ou padrão' },
            { name: 'map', detail: 'fn map<U>(self, f: fn(T) -> U) -> Result<U, E>', documentation: 'Mapeia sucesso' },
            { name: 'mapErr', detail: 'fn mapErr<F>(self, f: fn(E) -> F) -> Result<T, F>', documentation: 'Mapeia erro' },
            { name: 'andThen', detail: 'fn andThen<U>(self, f: fn(T) -> Result<U, E>) -> Result<U, E>', documentation: 'Encadeia result' },
            { name: 'orElse', detail: 'fn orElse(self, f: fn(E) -> Result<T, E>) -> Result<T, E>', documentation: 'Alternativa erro' },
            { name: 'contains', detail: 'fn contains(&self, value: &T) -> bool', documentation: 'Verifica valor' },
            { name: 'containsErr', detail: 'fn containsErr(&self, err: &E) -> bool', documentation: 'Verifica erro' },
        ]
    },
    {
        fullPath: 'core.file.File',
        shortName: 'File',
        members: [
            { name: 'open', detail: 'fn open(path: &str) -> Result<File, Error>', documentation: 'Abre arquivo' },
            { name: 'create', detail: 'fn create(path: &str) -> Result<File, Error>', documentation: 'Cria arquivo' },
            { name: 'readToString', detail: 'fn readToString(&self) -> Result<String, Error>', documentation: 'Lê arquivo como string' },
            { name: 'write', detail: 'fn write(&mut self, contents: &str) -> Result<usize, Error>', documentation: 'Escreve em arquivo' },
            { name: 'close', detail: 'fn close(self)', documentation: 'Fecha arquivo' },
            { name: 'exists', detail: 'fn exists(path: &str) -> bool', documentation: 'Verifica se arquivo existe' },
        ]
    },
    {
        fullPath: 'core.path.Path',
        shortName: 'Path',
        members: [
            { name: 'new', detail: 'fn new(path: &str) -> Path', documentation: 'Cria novo path' },
            { name: 'exists', detail: 'fn exists(&self) -> bool', documentation: 'Verifica se existe' },
            { name: 'isFile', detail: 'fn isFile(&self) -> bool', documentation: 'Verifica se é arquivo' },
            { name: 'isDir', detail: 'fn isDir(&self) -> bool', documentation: 'Verifica se é diretório' },
            { name: 'extension', detail: 'fn extension(&self) -> Option<String>', documentation: 'Retorna extensão' },
            { name: 'fileName', detail: 'fn fileName(&self) -> Option<String>', documentation: 'Retorna nome do arquivo' },
            { name: 'parent', detail: 'fn parent(&self) -> Option<Path>', documentation: 'Retorna diretório pai' },
            { name: 'join', detail: 'fn join(&self, other: &str) -> Path', documentation: 'Junta paths' },
        ]
    },
    {
        fullPath: 'core.process.Process',
        shortName: 'Process',
        members: [
            { name: 'spawn', detail: 'fn spawn(cmd: &str, args: Vec<String>) -> Child', documentation: 'Inicia processo' },
            { name: 'exit', detail: 'fn exit(code: i32)', documentation: 'Encerra processo' },
            { name: 'sleep', detail: 'fn sleep(ms: u64)', documentation: 'Aguarda milissegundos' },
            { name: 'envVars', detail: 'fn envVars() -> HashMap<String, String>', documentation: 'Variáveis de ambiente' },
            { name: 'args', detail: 'fn args() -> Vec<String>', documentation: 'Argumentos do programa' },
        ]
    },
    {
        fullPath: 'core.crypto.Hash',
        shortName: 'Hash',
        members: [
            { name: 'md5', detail: 'fn md5(data: &str) -> String', documentation: 'Hash MD5' },
            { name: 'sha256', detail: 'fn sha256(data: &str) -> String', documentation: 'Hash SHA256' },
            { name: 'sha512', detail: 'fn sha512(data: &str) -> String', documentation: 'Hash SHA512' },
            { name: 'base64Encode', detail: 'fn base64Encode(data: &str) -> String', documentation: 'Encode Base64' },
            { name: 'base64Decode', detail: 'fn base64Decode(data: &str) -> String', documentation: 'Decode Base64' },
            { name: 'uuid', detail: 'fn uuid() -> String', documentation: 'Gera UUID' },
        ]
    },
    {
        fullPath: 'core.function.Function',
        shortName: 'Function',
        members: [
            { name: 'call', detail: 'fn call<F, Args, R>(f: F, args: Args) -> R', documentation: 'Chama função' },
            { name: 'invoke', detail: 'fn invoke<F, R>(f: F) -> R', documentation: 'Invoca função' },
            { name: 'partial', detail: 'fn partial<F, A, B, R>(f: fn(A, B) -> R, a: A) -> fn(B) -> R', documentation: 'Aplica parcialmente' },
            { name: 'compose', detail: 'fn compose<A, B, C>(f: fn(B) -> C, g: fn(A) -> B) -> fn(A) -> C', documentation: 'Compoẽ funções' },
            { name: 'pipe', detail: 'fn pipe<A, B, C>(f: fn(A) -> B, g: fn(B) -> C) -> fn(A) -> C', documentation: 'Pipe de funções' },
            { name: 'curry', detail: 'fn curry<A, B, R>(f: fn(A, B) -> R) -> fn(A) -> fn(B) -> R', documentation: 'Curryfying' },
            { name: 'uncurry', detail: 'fn uncurry<A, B, R>(f: fn(A) -> fn(B) -> R) -> fn(A, B) -> R', documentation: 'Uncurryfying' },
            { name: 'memoize', detail: 'fn memoize<F, Args, R>(f: F) -> F', documentation: 'Memoização' },
            { name: 'once', detail: 'fn once<F, Args, R>(f: F) -> F', documentation: 'Executa apenas uma vez' },
            { name: 'debounce', detail: 'fn debounce<F, Args, R>(f: F, ms: u64) -> F', documentation: 'Debounce' },
            { name: 'throttle', detail: 'fn throttle<F, Args, R>(f: F, ms: u64) -> F', documentation: 'Throttle' },
        ]
    },
    {
        fullPath: 'core.async.Async',
        shortName: 'Async',
        members: [
            { name: 'sleep', detail: 'fn sleep(ms: u64) -> Future<void>', documentation: 'Aguarda ms' },
            { name: 'spawn', detail: 'fn spawn<F>(f: F) -> Task<F::Output>', documentation: 'Executa task' },
            { name: 'join', detail: 'fn join<T, E>(futures: Vec<Future<T>>) -> Future<Vec<Result<T, E>>>', documentation: 'Aguarda múltiplas' },
            { name: 'race', detail: 'fn race<T>(futures: Vec<Future<T>>) -> Future<T>', documentation: 'Primeira a resolver' },
            { name: 'all', detail: 'fn all<T>(futures: Vec<Future<T>>) -> Future<Vec<T>>', documentation: 'Todas completam' },
            { name: 'any', detail: 'fn any<T>(futures: Vec<Future<T>>) -> Future<T>', documentation: 'Qualquer completa' },
            { name: 'timeout', detail: 'fn timeout<T>(future: Future<T>, ms: u64) -> Future<Result<T, Error>>', documentation: 'Timeout' },
            { name: 'retry', detail: 'fn retry<T, E>(f: fn() -> Result<T, E>, attempts: u32) -> Future<T>', documentation: 'Tenta novamente' },
            { name: 'map', detail: 'fn map<F, T, U>(future: Future<T>, f: fn(T) -> U) -> Future<U>', documentation: 'Mapeia futuro' },
            { name: 'andThen', detail: 'fn andThen<F, T, U, E>(future: Future<Result<T, E>>, f: fn(T) -> Future<Result<U, E>>) -> Future<Result<U, E>>', documentation: 'Encadeia futuros' },
        ]
    },
    {
        fullPath: 'core.stream.Stream',
        shortName: 'Stream',
        members: [
            { name: 'fromIter', detail: 'fn fromIter<T>(iter: T) -> Stream<T::Item>', documentation: 'De iterador' },
            { name: 'fromFn', detail: 'fn fromFn<T>(f: fn() -> Option<T>) -> Stream<T>', documentation: 'De função' },
            { name: 'fromChannel', detail: 'fn fromChannel<T>(ch: Channel<T>) -> Stream<T>', documentation: 'De canal' },
            { name: 'map', detail: 'fn map<T, U>(stream: Stream<T>, f: fn(T) -> U) -> Stream<U>', documentation: 'Mapeia elementos' },
            { name: 'filter', detail: 'fn filter<T>(stream: Stream<T>, f: fn(&T) -> bool) -> Stream<T>', documentation: 'Filtra elementos' },
            { name: 'take', detail: 'fn take<T>(stream: Stream<T>, n: usize) -> Stream<T>', documentation: 'Take n elementos' },
            { name: 'skip', detail: 'fn skip<T>(stream: Stream<T>, n: usize) -> Stream<T>', documentation: 'Pula n elementos' },
            { name: 'flatten', detail: 'fn flatten<T>(stream: Stream<Stream<T>>) -> Stream<T>', documentation: 'Achata streams' },
            { name: 'fold', detail: 'fn fold<T, B>(stream: Stream<T>, init: B, f: fn(B, T) -> B) -> Future<B>', documentation: 'Reduz stream' },
            { name: 'collect', detail: 'fn collect<T>(stream: Stream<T>) -> Future<Vec<T>>', documentation: 'Coleta para Vec' },
            { name: 'forEach', detail: 'fn forEach<T>(stream: Stream<T>, f: fn(T)) -> Future<void>', documentation: 'Itera elementos' },
        ]
    },
    {
        fullPath: 'core.channel.Channel',
        shortName: 'Channel',
        members: [
            { name: 'new', detail: 'fn new<T>() -> (Sender<T>, Receiver<T>)', documentation: 'Cria canal' },
            { name: 'bounded', detail: 'fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>)', documentation: 'Canal limitado' },
            { name: 'unbounded', detail: 'fn unbounded<T>() -> (Sender<T>, Receiver<T>)', documentation: 'Canal ilimitado' },
            { name: 'send', detail: 'fn send(&self, value: T) -> Result<(), Error>', documentation: 'Envia valor' },
            { name: 'recv', detail: 'fn recv(&self) -> Result<T, Error>', documentation: 'Recebe valor' },
            { name: 'trySend', detail: 'fn trySend(&self, value: T) -> Result<(), TrySendError<T>>', documentation: 'Tenta enviar' },
            { name: 'tryRecv', detail: 'fn tryRecv(&self) -> Result<T, TryRecvError>', documentation: 'Tenta receber' },
            { name: 'close', detail: 'fn close(&mut self)', documentation: 'Fecha canal' },
            { name: 'isClosed', detail: 'fn isClosed(&self) -> bool', documentation: 'Verifica se fechado' },
        ]
    },
    {
        fullPath: 'core.mutex.Mutex',
        shortName: 'Mutex',
        members: [
            { name: 'new', detail: 'fn new<T>(value: T) -> Mutex<T>', documentation: 'Cria mutex' },
            { name: 'lock', detail: 'fn lock(&self) -> Guard<T>', documentation: 'Adquire lock' },
            { name: 'tryLock', detail: 'fn tryLock(&self) -> Option<Guard<T>>', documentation: 'Tenta adquirir lock' },
            { name: 'isLocked', detail: 'fn isLocked(&self) -> bool', documentation: 'Verifica se bloqueado' },
            { name: 'get', detail: 'fn get(&self) -> Option<&T>', documentation: 'Obtém valor' },
        ]
    },
    {
        fullPath: 'core.thread.Thread',
        shortName: 'Thread',
        members: [
            { name: 'spawn', detail: 'fn spawn(f: fn()) -> Thread', documentation: 'Cria thread' },
            { name: 'current', detail: 'fn current() -> Thread', documentation: 'Thread atual' },
            { name: 'sleep', detail: 'fn sleep(duration: Duration)', documentation: 'Dorme por duração' },
            { name: 'yieldNow', detail: 'fn yieldNow()', documentation: 'cede execução' },
            { name: 'id', detail: 'fn id() -> ThreadId', documentation: 'ID da thread' },
            { name: 'name', detail: 'fn name(&self) -> Option<&str>', documentation: 'Nome da thread' },
            { name: 'panic', detail: 'fn panic(msg: &str)', documentation: 'Pânico na thread' },
        ]
    },
    {
        fullPath: 'core.random.Random',
        shortName: 'Random',
        members: [
            { name: 'new', detail: 'fn new() -> Random', documentation: 'Cria gerador' },
            { name: 'seed', detail: 'fn seed(&mut self, seed: u64)', documentation: 'Define seed' },
            { name: 'nextU8', detail: 'fn nextU8(&mut self) -> u8', documentation: 'Próximo u8' },
            { name: 'nextU32', detail: 'fn nextU32(&mut self) -> u32', documentation: 'Próximo u32' },
            { name: 'nextU64', detail: 'fn nextU64(&mut self) -> u64', documentation: 'Próximo u64' },
            { name: 'nextF32', detail: 'fn nextF32(&mut self) -> f32', documentation: 'Próximo f32' },
            { name: 'nextF64', detail: 'fn nextF64(&mut self) -> f64', documentation: 'Próximo f64' },
            { name: 'genRange', detail: 'fn genRange<T>(&mut self, range: Range<T>) -> T', documentation: 'Gera no range' },
            { name: 'gen', detail: 'fn gen<T>(&mut self) -> T', documentation: 'Gera valor aleatório' },
            { name: 'shuffle', detail: 'fn shuffle<T>(&mut self, slice: &mut [T])', documentation: 'Embaralha' },
            { name: 'sample', detail: 'fn sample<T>(&self, slice: &[T], count: usize) -> Vec<T>', documentation: 'Amostra elementos' },
        ]
    },
    {
        fullPath: 'core.regex.Regex',
        shortName: 'Regex',
        members: [
            { name: 'new', detail: 'fn new(pattern: &str) -> Result<Regex, Error>', documentation: 'Cria regex' },
            { name: 'isMatch', detail: 'fn isMatch(&self, text: &str) -> bool', documentation: 'Verifica match' },
            { name: 'find', detail: 'fn find(&self, text: &str) -> Option<Match>', documentation: 'Encontra match' },
            { name: 'findAll', detail: 'fn findAll(&self, text: &str) -> Vec<Match>', documentation: 'Encontra todos' },
            { name: 'captures', detail: 'fn captures(&self, text: &str) -> Option<Captures>', documentation: 'Capturas' },
            { name: 'replace', detail: 'fn replace(&self, text: &str, repl: &str) -> String', documentation: 'Substitui' },
            { name: 'split', detail: 'fn split(&self, text: &str) -> Vec<String>', documentation: 'Divide' },
            { name: 'captureNames', detail: 'fn captureNames(&self) -> Vec<Option<&str>>', documentation: 'Nomes de captura' },
        ]
    },
    {
        fullPath: 'core.url.Url',
        shortName: 'Url',
        members: [
            { name: 'parse', detail: 'fn parse(s: &str) -> Result<Url, Error>', documentation: 'Parseia URL' },
            { name: 'new', detail: 'fn new() -> Url', documentation: 'Cria URL vazia' },
            { name: 'scheme', detail: 'fn scheme(&self) -> &str', documentation: 'Retorna scheme' },
            { name: 'host', detail: 'fn host(&self) -> Option<&str>', documentation: 'Retorna host' },
            { name: 'port', detail: 'fn port(&self) -> Option<u16>', documentation: 'Retorna porta' },
            { name: 'path', detail: 'fn path(&self) -> &str', documentation: 'Retorna path' },
            { name: 'query', detail: 'fn query(&self) -> Option<&str>', documentation: 'Retorna query' },
            { name: 'fragment', detail: 'fn fragment(&self) -> Option<&str>', documentation: 'Retorna fragment' },
            { name: 'setScheme', detail: 'fn setScheme(&mut self, scheme: &str) -> Result<(), Error>', documentation: 'Define scheme' },
            { name: 'setHost', detail: 'fn setHost(&mut self, host: &str) -> Result<(), Error>', documentation: 'Define host' },
            { name: 'setPort', detail: 'fn setPort(&mut self, port: u16)', documentation: 'Define porta' },
            { name: 'join', detail: 'fn join(&self, path: &str) -> Result<Url, Error>', documentation: 'Junta URLs' },
            { name: 'toString', detail: 'fn toString(&self) -> String', documentation: 'Converte para string' },
        ]
    },
    {
        fullPath: 'core.uuid.Uuid',
        shortName: 'Uuid',
        members: [
            { name: 'newV4', detail: 'fn newV4() -> Uuid', documentation: 'Gera UUID v4' },
            { name: 'newV5', detail: 'fn newV5(namespace: &Uuid, name: &str) -> Uuid', documentation: 'Gera UUID v5' },
            { name: 'nil', detail: 'fn nil() -> Uuid', documentation: 'UUID nil' },
            { name: 'parse', detail: 'fn parse(s: &str) -> Option<Uuid>', documentation: 'Parseia UUID' },
            { name: 'toString', detail: 'fn toString(&self) -> String', documentation: 'Converte para string' },
            { name: 'version', detail: 'fn version(&self) -> UuidVersion', documentation: 'Retorna versão' },
            { name: 'variant', detail: 'fn variant(&self) -> UuidVariant', documentation: 'Retorna variante' },
        ]
    },
    {
        fullPath: 'core.math.Math',
        shortName: 'Math',
        members: [
            { name: 'abs', detail: 'fn abs<T: Abs>(n: T) -> T', documentation: 'Valor absoluto' },
            { name: 'ceil', detail: 'fn ceil(n: f64) -> f64', documentation: 'Teto' },
            { name: 'floor', detail: 'fn floor(n: f64) -> f64', documentation: 'Piso' },
            { name: 'round', detail: 'fn round(n: f64) -> f64', documentation: 'Arredonda' },
            { name: 'sqrt', detail: 'fn sqrt(n: f64) -> f64', documentation: 'Raiz quadrada' },
            { name: 'pow', detail: 'fn pow(base: f64, exp: f64) -> f64', documentation: 'Potência' },
            { name: 'exp', detail: 'fn exp(n: f64) -> f64', documentation: 'Exponencial' },
            { name: 'ln', detail: 'fn ln(n: f64) -> f64', documentation: 'Log natural' },
            { name: 'log', detail: 'fn log(base: f64, n: f64) -> f64', documentation: 'Log base' },
            { name: 'log2', detail: 'fn log2(n: f64) -> f64', documentation: 'Log base 2' },
            { name: 'log10', detail: 'fn log10(n: f64) -> f64', documentation: 'Log base 10' },
            { name: 'sin', detail: 'fn sin(n: f64) -> f64', documentation: 'Seno' },
            { name: 'cos', detail: 'fn cos(n: f64) -> f64', documentation: 'Cosseno' },
            { name: 'tan', detail: 'fn tan(n: f64) -> f64', documentation: 'Tangente' },
            { name: 'asin', detail: 'fn asin(n: f64) -> f64', documentation: 'Arco seno' },
            { name: 'acos', detail: 'fn acos(n: f64) -> f64', documentation: 'Arco cosseno' },
            { name: 'atan', detail: 'fn atan(n: f64) -> f64', documentation: 'Arco tangente' },
            { name: 'atan2', detail: 'fn atan2(y: f64, x: f64) -> f64', documentation: 'Arco tangente 2' },
            { name: 'sinh', detail: 'fn sinh(n: f64) -> f64', documentation: 'Seno hiperbólico' },
            { name: 'cosh', detail: 'fn cosh(n: f64) -> f64', documentation: 'Cosseno hiperbólico' },
            { name: 'tanh', detail: 'fn tanh(n: f64) -> f64', documentation: 'Tangente hiperbólica' },
            { name: 'clamp', detail: 'fn clamp(n: f64, min: f64, max: f64) -> f64', documentation: 'Limita valor' },
            { name: 'max', detail: 'fn max<T: PartialOrd>(a: T, b: T) -> T', documentation: 'Máximo' },
            { name: 'min', detail: 'fn min<T: PartialOrd>(a: T, b: T) -> T', documentation: 'Mínimo' },
        ]
    },
    {
        fullPath: 'core.hash.Hash',
        shortName: 'Hash',
        members: [
            { name: 'hash', detail: 'fn hash<T: Hash>(value: &T) -> u64', documentation: 'Hash de valor' },
            { name: 'hashSlice', detail: 'fn hashSlice<T: Hash>(slice: &[T]) -> u64', documentation: 'Hash de slice' },
            { name: 'hasher', detail: 'fn hasher() -> DefaultHasher', documentation: 'Hasher padrão' },
            { name: 'BuildHasher', detail: 'fn buildHasher() -> BuildHasherDefault', documentation: 'BuildHasher' },
        ]
    },
    {
        fullPath: 'core.unsafe.Unsafe',
        shortName: 'Unsafe',
        members: [
            { name: 'cast', detail: 'fn cast<T, U>(value: T) -> U', documentation: 'Cast de tipo' },
            { name: 'castPtr', detail: 'fn castPtr<T, U>(ptr: *const T) -> *const U', documentation: 'Cast de ponteiro' },
            { name: 'ptrRead', detail: 'fn ptrRead<T>(ptr: *const T) -> T', documentation: 'Lê de ponteiro' },
            { name: 'ptrWrite', detail: 'fn ptrWrite<T>(ptr: *mut T, value: T)', documentation: 'Escreve em ponteiro' },
            { name: 'ptrSwap', detail: 'fn ptrSwap<T>(a: *mut T, b: *mut T)', documentation: 'Troca valores' },
            { name: 'zeroed', detail: 'fn zeroed<T>() -> T', documentation: 'Zera memória' },
            { name: 'uninitialized', detail: 'fn uninitialized<T>() -> T', documentation: 'Não inicializado' },
            { name: 'transmute', detail: 'fn transmute<T, U>(value: T) -> U', documentation: 'Transmuta valor' },
            { name: 'copy', detail: 'fn copy<T>(src: *const T, dst: *mut T, count: usize)', documentation: 'Copia memória' },
            { name: 'copyNonOverlapping', detail: 'fn copyNonOverlapping<T>(src: *const T, dst: *mut T, count: usize)', documentation: 'Copia não sobreposta' },
            { name: 'write', detail: 'fn write<T>(dst: *mut T, src: T)', documentation: 'Escreve valor' },
            { name: 'read', detail: 'fn read<T>(src: *const T) -> T', documentation: 'Lê valor' },
        ]
    },
    {
        fullPath: 'core.marker.Marker',
        shortName: 'Marker',
        members: [
            { name: 'Send', detail: 'trait Send', documentation: 'Marker para thread safety' },
            { name: 'Sync', detail: 'trait Sync', documentation: 'Marker para compartilhamento' },
            { name: 'Copy', detail: 'trait Copy', documentation: 'Marker para cópia bitwise' },
            { name: 'Clone', detail: 'trait Clone', documentation: 'Marker para clonagem' },
            { name: 'Default', detail: 'trait Default', documentation: 'Marker para valor padrão' },
            { name: 'PartialEq', detail: 'trait PartialEq', documentation: 'Marker para igualdade' },
            { name: 'PartialOrd', detail: 'trait PartialOrd', documentation: 'Marker para ordenação' },
            { name: 'Eq', detail: 'trait Eq', documentation: 'Marker para equivalência' },
            { name: 'Ord', detail: 'trait Ord', documentation: 'Marker para ordenação total' },
            { name: 'Hash', detail: 'trait Hash', documentation: 'Marker para hash' },
            { name: 'Display', detail: 'trait Display', documentation: 'Marker para display' },
            { name: 'Debug', detail: 'trait Debug', documentation: 'Marker para debug' },
            { name: 'From', detail: 'trait From<T>', documentation: 'Marker para conversão de' },
            { name: 'Into', detail: 'trait Into<U>', documentation: 'Marker para conversão para' },
            { name: 'AsRef', detail: 'trait AsRef<T>', documentation: 'Marker para referência' },
            { name: 'AsMut', detail: 'trait AsMut<T>', documentation: 'Marker para mutação' },
            { name: 'Deref', detail: 'trait Deref', documentation: 'Marker para dereferência' },
            { name: 'DerefMut', detail: 'trait DerefMut', documentation: 'Marker para deref mut' },
            { name: 'Index', detail: 'trait Index<I>', documentation: 'Marker para indexação' },
            { name: 'IndexMut', detail: 'trait IndexMut<I>', documentation: 'Marker para index mut' },
            { name: 'Drop', detail: 'trait Drop', documentation: 'Marker para cleanup' },
            { name: 'Sized', detail: 'trait Sized', documentation: 'Marker para tamanho conhecido' },
        ]
    },
    {
        fullPath: 'core.iterator.Iterator',
        shortName: 'Iterator',
        members: [
            { name: 'next', detail: 'fn next(&mut self) -> Option<Self::Item>', documentation: 'Próximo elemento' },
            { name: 'count', detail: 'fn count(self) -> usize', documentation: 'Conta elementos' },
            { name: 'last', detail: 'fn last(self) -> Option<Self::Item>', documentation: 'Último elemento' },
            { name: 'nth', detail: 'fn nth(&mut self, n: usize) -> Option<Self::Item>', documentation: 'N-ésimo elemento' },
            { name: 'stepBy', detail: 'fn stepBy(&self, step: usize) -> StepBy<Self>', documentation: 'Step iterator' },
            { name: 'chain', detail: 'fn chain<I>(self, other: I) -> Chain<Self, I::IntoIter>', documentation: 'Concatena iteradores' },
            { name: 'zip', detail: 'fn zip<I>(self, other: I) -> Zip<Self, I::IntoIter>', documentation: 'Zip iteradores' },
            { name: 'map', detail: 'fn map<B>(self, f: fn(A) -> B) -> Map<Self, F>', documentation: 'Mapeia elementos' },
            { name: 'filter', detail: 'fn filter<P>(self, predicate: P) -> Filter<Self, P>', documentation: 'Filtra elementos' },
            { name: 'enumerate', detail: 'fn enumerate(self) -> Enumerate<Self>', documentation: 'Enumera elementos' },
            { name: 'peekable', detail: 'fn peekable(self) -> Peekable<Self>', documentation: 'Iterador com peek' },
            { name: 'fuse', detail: 'fn fuse(self) -> Fuse<Self>', documentation: 'Para no primeiro None' },
            { name: 'rev', detail: 'fn rev(self) -> Rev<Self>', documentation: 'Inverte iterador' },
            { name: 'skip', detail: 'fn skip(self, n: usize) -> Skip<Self>', documentation: 'Pula n elementos' },
            { name: 'take', detail: 'fn take(self, n: usize) -> Take<Self>', documentation: 'Pega n elementos' },
            { name: 'cloned', detail: 'fn cloned(self) -> Cloned<Self>', documentation: 'Clona elementos' },
            { name: 'cycle', detail: 'fn cycle(self) -> Cycle<Self>', documentation: 'Repete ciclicamente' },
            { name: 'sum', detail: 'fn sum<S>(self) -> S', documentation: 'Soma elementos' },
            { name: 'product', detail: 'fn product<P>(self) -> P', documentation: 'Produto de elementos' },
            { name: 'find', detail: 'fn find<P>(&mut self, predicate: P) -> Option<Self::Item>', documentation: 'Encontra elemento' },
            { name: 'position', detail: 'fn position<P>(&mut self, predicate: P) -> Option<usize>', documentation: 'Posição de elemento' },
            { name: 'all', detail: 'fn all<P>(&mut self, predicate: P) -> bool', documentation: 'Todos satisfazem' },
            { name: 'any', detail: 'fn any<P>(&mut self, predicate: P) -> bool', documentation: 'Algum satisfaz' },
            { name: 'forEach', detail: 'fn forEach<F>(&mut self, f: F)', documentation: 'Executa para cada' },
            { name: 'fold', detail: 'fn fold<B, F>(self, init: B, f: F) -> B', documentation: 'Reduz elementos' },
        ]
    },
    {
        fullPath: 'core.option.OptionExt',
        shortName: 'OptionExt',
        members: [
            { name: 'and', detail: 'fn and<T>(self, opt: Option<T>) -> Option<T>', documentation: 'AND lógico' },
            { name: 'andThen', detail: 'fn andThen<T, F>(self, f: F) -> Option<T> where F: FnOnce(&Self::Item) -> Option<T>', documentation: 'And then' },
            { name: 'filter', detail: 'fn filter<P>(self, predicate: P) -> Self where P: FnOnce(&Self::Item)', documentation: 'Filtra' },
            { name: 'flatten', detail: 'fn flatten<T>(self) -> Option<T> where Self: Option<Option<T>>', documentation: 'Achata' },
            { name: 'insert', detail: 'fn insert(&mut self, value: Self::Item) -> &mut Self::Item', documentation: 'Insere valor' },
            { name: 'getOrInsert', detail: 'fn getOrInsert(&mut self, default: Self::Item) -> &mut Self::Item', documentation: 'Obtém ou insere' },
            { name: 'getOrInsertWith', detail: 'fn getOrInsertWith<F>(&mut self, f: F) -> &mut Self::Item where F: FnOnce() -> Self::Item', documentation: 'Obtém ou calcula' },
            { name: 'replace', detail: 'fn replace(&mut self, value: Self::Item) -> Option<Self::Item>', documentation: 'Substitui' },
            { name: 'transpose', detail: 'fn transpose<T, E>(self) -> Result<Option<T>, E> where Self: Option<Result<T, E>>', documentation: 'Transpõe' },
            { name: 'zip', detail: 'fn zip<T>(self, other: Option<T>) -> Option<(Self::Item, T)>', documentation: 'Zip options' },
            { name: 'unzip', detail: 'fn unzip<A, B>(self) -> (Option<A>, Option<B>) where Self: Option<(A, B)>', documentation: 'Unzip' },
        ]
    },
    {
        fullPath: 'core.result.ResultExt',
        shortName: 'ResultExt',
        members: [
            { name: 'map', detail: 'fn map<U, F>(self, op: F) -> Result<U, E> where F: FnOnce(T) -> U', documentation: 'Mapeia Ok' },
            { name: 'mapOr', detail: 'fn mapOr<U, F>(self, default: U, op: F) -> U where F: FnOnce(T) -> U', documentation: 'Mapeia ou default' },
            { name: 'mapOrElse', detail: 'fn mapOrElse<U, D, F>(self, default: D, op: F) -> U where F: FnOnce(T) -> U, D: FnOnce(E) -> U', documentation: 'Mapeia ou calcula' },
            { name: 'mapErr', detail: 'fn mapErr<F, O>(self, op: O) -> Result<T, F> where O: FnOnce(E) -> F', documentation: 'Mapeia erro' },
            { name: 'and', detail: 'fn and<T>(self, res: Result<T, E>) -> Result<T, E>', documentation: 'AND lógico' },
            { name: 'andThen', detail: 'fn andThen<U, F>(self, op: F) -> Result<U, E> where F: FnOnce(T) -> Result<U, E>', documentation: 'And then' },
            { name: 'or', detail: 'fn or<T>(self, res: Result<T, F>) -> Result<T, F>', documentation: 'OR lógico' },
            { name: 'orElse', detail: 'fn orElse<F, O>(self, op: O) -> Result<T, F> where O: FnOnce(E) -> Result<T, F>', documentation: 'Or else' },
            { name: 'unwrapOr', detail: 'fn unwrapOr(self, default: T) -> T', documentation: 'Unwrap ou default' },
            { name: 'unwrapOrElse', detail: 'fn unwrapOrElse<F>(self, op: F) -> T where F: FnOnce(E) -> T', documentation: 'Unwrap ou calcula' },
            { name: 'transpose', detail: 'fn transpose<T>(self) -> Result<Option<T>, E> where Self: Result<Option<T>, E>', documentation: 'Transpõe' },
            { name: 'flatten', detail: 'fn flatten<T, E>(self) -> Result<T, E> where Self: Result<Result<T, E>, E>', documentation: 'Achata' },
            { name: 'intoOk', detail: 'fn intoOk(self) -> T', documentation: 'Converte para Ok' },
            { name: 'intoErr', detail: 'fn intoErr(self) -> E', documentation: 'Converte para Err' },
        ]
    },
    {
        fullPath: 'core.panic.Panic',
        shortName: 'Panic',
        members: [
            { name: 'panic', detail: 'fn panic(msg: &str) -> !', documentation: 'Pânico com mensagem' },
            { name: 'panicAny', detail: 'fn panicAny<T: Any + Send>(info: T) -> !', documentation: 'Pânico com info' },
            { name: 'catchUnwind', detail: 'fn catchUnwind<F, T>(f: F) -> Result<T, Box<dyn Any + Send>>', documentation: 'Captura pânico' },
            { name: 'resumeUnwind', detail: 'fn resumeUnwind(payload: Box<dyn Any + Send>) -> !', documentation: 'Resume pânico' },
            { name: 'Assert', detail: 'fn Assert(cond: bool, msg: &str)', documentation: 'Assert customizado' },
            { name: 'assertEq', detail: 'fn assertEq<T: PartialEq + Debug>(a: T, b: T)', documentation: 'Assert igualdade' },
            { name: 'assertNe', detail: 'fn assertNe<T: PartialEq + Debug>(a: T, b: T)', documentation: 'Assert desigualdade' },
            { name: 'assertGt', detail: 'fn assertGt<T: PartialOrd + Debug>(a: T, b: T)', documentation: 'Assert maior que' },
            { name: 'assertLt', detail: 'fn assertLt<T: PartialOrd + Debug>(a: T, b: T)', documentation: 'Assert menor que' },
            { name: 'assertMatch', detail: 'fn assertMatch<T: Match>(value: &T, pattern: &str)', documentation: 'Assert match' },
            { name: 'unreachable', detail: 'fn unreachable() -> !', documentation: 'Código inalcançável' },
            { name: 'unimplemented', detail: 'fn unimplemented() -> !', documentation: 'Não implementado' },
        ]
    },
];

const typeAnnotations = [
    { name: 'Getter', detail: '@Getter', documentation: 'Gera getter para campos' },
    { name: 'Setter', detail: '@Setter', documentation: 'Gera setter para campos' },
    { name: 'Constructor', detail: '@Constructor', documentation: 'Gera construtor' },
    { name: 'Data', detail: '@Data', documentation: 'Gera getters, setters, equals, hashCode' },
    { name: 'Builder', detail: '@Builder', documentation: 'Gera builder pattern' },
    { name: 'AllArgsConstructor', detail: '@AllArgsConstructor', documentation: 'Gera construtor com todos args' },
    { name: 'NoArgsConstructor', detail: '@NoArgsConstructor', documentation: 'Gera construtor vazio' },
    { name: 'ToString', detail: '@ToString', documentation: 'Gera método toString' },
    { name: 'Equals', detail: '@Equals', documentation: 'Gera método equals' },
    { name: 'HashCode', detail: '@HashCode', documentation: 'Gera método hashCode' },
    { name: 'Deprecated', detail: '@Deprecated', documentation: 'Marca como obsoleto' },
    { name: 'Override', detail: '@Override', documentation: 'Marca como sobrescrito' },
    { name: 'Test', detail: '@Test', documentation: 'Marca método como teste' },
    { name: 'Before', detail: '@Before', documentation: 'Executa antes de cada teste' },
    { name: 'After', detail: '@After', documentation: 'Executa depois de cada teste' },
    { name: 'BeforeAll', detail: '@BeforeAll', documentation: 'Executa antes de todos testes' },
    { name: 'AfterAll', detail: '@AfterAll', documentation: 'Executa depois de todos testes' },
];

const httpDecorators = [
    { name: 'Get', detail: '@Get("/path")', documentation: 'Rota GET HTTP' },
    { name: 'Post', detail: '@Post("/path")', documentation: 'Rota POST HTTP' },
    { name: 'Put', detail: '@Put("/path")', documentation: 'Rota PUT HTTP' },
    { name: 'Delete', detail: '@Delete("/path")', documentation: 'Rota DELETE HTTP' },
    { name: 'Patch', detail: '@Patch("/path")', documentation: 'Rota PATCH HTTP' },
    { name: 'Head', detail: '@Head("/path")', documentation: 'Rota HEAD HTTP' },
    { name: 'Options', detail: '@Options("/path")', documentation: 'Rota OPTIONS HTTP' },
    { name: 'Middleware', detail: '@Middleware', documentation: 'Middleware HTTP' },
    { name: 'Guard', detail: '@Guard', documentation: 'Guard de rota' },
    { name: 'Route', detail: '@Route("/path")', documentation: 'Rota genérica' },
];

const builtInFunctions = [
    { name: 'println', detail: 'fn println(msg: String)', documentation: 'Imprime com nova linha' },
    { name: 'print', detail: 'fn print(msg: String)', documentation: 'Imprime sem nova linha' },
    { name: 'panic', detail: 'fn panic(msg: String) -> !', documentation: 'Pânico com mensagem' },
    { name: 'assert', detail: 'fn assert(cond: bool)', documentation: 'Assert booleano' },
    { name: 'assertEq', detail: 'fn assertEq<T>(a: T, b: T)', documentation: 'Assert igualdade' },
    { name: 'assertNe', detail: 'fn assertNe<T>(a: T, b: T)', documentation: 'Assert desigualdade' },
    { name: 'dbg', detail: 'fn dbg<T>(expr: T) -> T', documentation: 'Debug expression' },
    { name: 'todo', detail: 'fn todo()', documentation: 'Marca como TODO' },
    { name: 'unimplemented', detail: 'fn unimplemented() -> !', documentation: 'Não implementado' },
    { name: 'unreachable', detail: 'fn unreachable() -> !', documentation: 'Código inalcançável' },
    { name: 'sizeOf', detail: 'fn sizeOf<T>() -> usize', documentation: 'Tamanho de tipo' },
    { name: 'typeOf', detail: 'fn typeOf<T>() -> &str', documentation: 'Nome do tipo' },
];

export function activate(context: vscode.ExtensionContext) {
    const selector: vscode.DocumentSelector = { language: 'lexicon' };

    // Completion Provider
    const completionProvider = vscode.languages.registerCompletionItemProvider(
        selector,
        {
            provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
                const line = document.lineAt(position.line).text;
                const beforeCursor = line.substring(0, position.character);
                const afterCursor = line.substring(position.character);
                
                const completionItems: vscode.CompletionItem[] = [];
                
                // Keywords
                const keywords = [
                    'fn', 'let', 'mut', 'const', 'var', 'class', 'struct', 'enum', 'pub', 'private',
                    'protected', 'static', 'async', 'await', 'if', 'else', 'match', 'for',
                    'while', 'loop', 'return', 'break', 'continue', 'import', 'module',
                    'type', 'trait', 'impl', 'self', 'super', 'crate', 'mod', 'use',
                    'true', 'false', 'nil', 'try', 'catch', 'throw', 'throws', 'as', 'is',
                    'where', 'yield', 'in', 'unsafe', 'virtual', 'override', 'final', 'abstract',
                    'extends', 'implements', 'instanceof', 'package', 'synchronized', 'transient',
                    'volatile', 'native', 'strictfp', 'sealed', 'nonsealed', 'permits', 'provides', 'with'
                ];
                
                for (const kw of keywords) {
                    const item = new vscode.CompletionItem(kw, vscode.CompletionItemKind.Keyword);
                    item.detail = `keyword: ${kw}`;
                    completionItems.push(item);
                }
                
                // Types
                const types = [
                    'String', 'i8', 'i16', 'i32', 'i64', 'i128', 'u8', 'u16', 'u32', 'u64', 'u128',
                    'f32', 'f64', 'bool', 'char', 'byte', 'void', 'usize', 'isize',
                    'List', 'Map', 'Set', 'Option', 'Result', 'Vec', 'Box', 'Rc', 'Arc', 
                    'Cell', 'RefCell', 'HashMap', 'HashSet', 'StringBuffer', 'StringBuilder',
                    'Path', 'PathBuf', 'File', 'BufReader', 'BufWriter', 'IpAddr', 'SocketAddr',
                    'Duration', 'Instant', 'SystemTime', 'DateTime', 'NaiveDate', 'NaiveTime',
                    'NonZeroU8', 'NonZeroU16', 'NonZeroU32', 'NonZeroU64', 'NonZeroU128',
                    'NonZeroI8', 'NonZeroI16', 'NonZeroI32', 'NonZeroI64', 'NonZeroI128',
                    'Cow', 'OnceCell', 'Cell', 'UnsafeCell', 'PhantomData'
                ];
                
                for (const t of types) {
                    const item = new vscode.CompletionItem(t, vscode.CompletionItemKind.TypeParameter);
                    item.detail = `type: ${t}`;
                    item.documentation = new vscode.MarkdownString(`**${t}** - Built-in type in Lexicon`);
                    completionItems.push(item);
                }
                
                // Built-in functions
                for (const fn of builtInFunctions) {
                    const item = new vscode.CompletionItem(fn.name, vscode.CompletionItemKind.Function);
                    item.detail = fn.detail;
                    item.documentation = fn.documentation;
                    completionItems.push(item);
                }
                
                // Import completions
                if (beforeCursor.includes('import')) {
                    for (const mod of builtinModules) {
                        const item = new vscode.CompletionItem(mod.fullPath, vscode.CompletionItemKind.Module);
                        item.detail = `import ${mod.fullPath}`;
                        item.documentation = new vscode.MarkdownString(
                            `**${mod.shortName}**\n\n${mod.members.map(m => `- \`${m.name}\`: ${m.documentation}`).join('\n')}`
                        );
                        completionItems.push(item);
                    }
                }
                
                // Module member completions
                if (beforeCursor.includes('::') || beforeCursor.endsWith('.')) {
                    for (const mod of builtinModules) {
                        for (const member of mod.members) {
                            const item = new vscode.CompletionItem(member.name, vscode.CompletionItemKind.Function);
                            item.detail = member.detail;
                            item.documentation = member.documentation;
                            item.insertText = `${mod.shortName}::${member.name}`;
                            completionItems.push(item);
                        }
                    }
                }
                
                // Annotations
                for (const ann of typeAnnotations) {
                    const item = new vscode.CompletionItem(ann.name, vscode.CompletionItemKind.Interface);
                    item.detail = ann.detail;
                    item.documentation = ann.documentation;
                    completionItems.push(item);
                }
                
                // HTTP Decorators
                for (const dec of httpDecorators) {
                    const item = new vscode.CompletionItem(dec.name, vscode.CompletionItemKind.EnumMember);
                    item.detail = dec.detail;
                    item.documentation = dec.documentation;
                    completionItems.push(item);
                }
                
                // Module completions
                for (const mod of builtinModules) {
                    const item = new vscode.CompletionItem(mod.shortName, vscode.CompletionItemKind.Module);
                    item.detail = mod.fullPath;
                    item.documentation = new vscode.MarkdownString(
                        `**${mod.fullPath}**\n\nImporte com: \`import ${mod.fullPath};\`\n\n**Membros:**\n${mod.members.map(m => `- \`${m.name}\`: ${m.documentation}`).join('\n')}`
                    );
                    completionItems.push(item);
                    
                    for (const member of mod.members) {
                        const memberItem = new vscode.CompletionItem(member.name, vscode.CompletionItemKind.Method);
                        memberItem.detail = member.detail;
                        memberItem.documentation = member.documentation;
                        memberItem.insertText = member.name;
                        completionItems.push(memberItem);
                    }
                }
                
                return completionItems;
            },
            resolveCompletionItem(item: vscode.CompletionItem) {
                return item;
            }
        },
        ...['.', ':', ' ', '\n', '::', '@']
    );

    // Hover Provider
    const hoverProvider = vscode.languages.registerHoverProvider(selector, {
        provideHover(document: vscode.TextDocument, position: vscode.Position) {
            const range = document.getWordRangeAtPosition(position);
            const word = document.getText(range);
            
            // Check modules
            for (const mod of builtinModules) {
                if (word === mod.shortName || word === mod.fullPath) {
                    return new vscode.Hover(
                        new vscode.MarkdownString(
                            `**${mod.fullPath}**\n\nMódulo do core\n\n**Membros:**\n${mod.members.map(m => `- \`${m.name}\`: ${m.documentation}`).join('\n')}`
                        ),
                        range
                    );
                }
                
                for (const member of mod.members) {
                    if (word === member.name) {
                        return new vscode.Hover(
                            new vscode.MarkdownString(
                                `**${mod.shortName}::${member.name}**\n\n\`${member.detail}\`\n\n${member.documentation}`
                            ),
                            range
                        );
                    }
                }
            }
            
            // Check types
            const types: { [key: string]: string } = {
                'String': 'String - Texto UTF-8',
                'i8': 'i8 - Inteiro signed 8-bit',
                'i16': 'i16 - Inteiro signed 16-bit',
                'i32': 'i32 - Inteiro signed 32-bit',
                'i64': 'i64 - Inteiro signed 64-bit',
                'i128': 'i128 - Inteiro signed 128-bit',
                'u8': 'u8 - Inteiro unsigned 8-bit',
                'u16': 'u16 - Inteiro unsigned 16-bit',
                'u32': 'u32 - Inteiro unsigned 32-bit',
                'u64': 'u64 - Inteiro unsigned 64-bit',
                'u128': 'u128 - Inteiro unsigned 128-bit',
                'f32': 'f32 - Float 32-bit',
                'f64': 'f64 - Float 64-bit',
                'bool': 'bool - Booleano (true/false)',
                'char': 'char - Caractere Unicode',
                'void': 'void - Tipo vazio',
                'List': 'List<T> - Lista dinâmica',
                'Map': 'Map<K, V> - Mapa hash',
                'Set': 'Set<T> - Conjunto hash',
                'Option': 'Option<T> - Tipo opcional (Some/None)',
                'Result': 'Result<T, E> - Resultado (Ok/Err)',
                'Vec': 'Vec<T> - Vetor',
                'StringBuffer': 'StringBuffer - Buffer de string',
            };
            
            if (types[word]) {
                return new vscode.Hover(
                    new vscode.MarkdownString(`**${word}**: ${types[word]}`),
                    range
                );
            }
            
            // Check keywords
            const keywords: { [key: string]: string } = {
                'fn': 'fn - Declaração de função',
                'let': 'let - Declaração de variável imutável',
                'mut': 'mut - Variável mutável',
                'const': 'const - Constante',
                'class': 'class - Definição de classe',
                'struct': 'struct - Definição de estrutura',
                'enum': 'enum - Definição de enum',
                'trait': 'trait - Definição de trait',
                'impl': 'impl - Implementação de trait',
                'pub': 'pub - Modificador público',
                'private': 'private - Modificador privado',
                'if': 'if - Condicional',
                'else': 'else - Senão',
                'match': 'match - Pattern matching',
                'for': 'for - Loop for',
                'while': 'while - Loop while',
                'loop': 'loop - Loop infinito',
                'return': 'return - Retorno de função',
                'break': 'break - Sai do loop',
                'continue': 'continue - Próxima iteração',
                'import': 'import - Importa módulo',
                'async': 'async - Função assíncrona',
                'await': 'await - Espera futures',
                'try': 'try - Bloco de try-catch',
                'catch': 'catch - Captura erro',
                'throw': 'throw - Lança exceção',
                'as': 'as - Cast de tipo',
                'self': 'self - Referência ao objeto',
                'super': 'super - Classe pai',
                'true': 'true - Booleano verdadeiro',
                'false': 'false - Booleano falso',
                'None': 'None - Valor ausente (Option)',
                'Some': 'Some - Valor presente (Option)',
                'Ok': 'Ok - Sucesso (Result)',
                'Err': 'Err - Erro (Result)',
            };
            
            if (keywords[word]) {
                return new vscode.Hover(
                    new vscode.MarkdownString(keywords[word]),
                    range
                );
            }
            
            return null;
        }
    });

    // Definition Provider
    const definitionProvider = vscode.languages.registerDefinitionProvider(selector, {
        provideDefinition(document: vscode.TextDocument, position: vscode.Position) {
            const range = document.getWordRangeAtPosition(position);
            const word = document.getText(range);
            
            // Find function definitions
            const text = document.getText();
            const lines = text.split('\n');
            
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i];
                if (line.includes(`fn ${word}`) || line.includes(`pub fn ${word}`)) {
                    const lineRange = new vscode.Range(i, 0, i, line.length);
                    return new vscode.Location(document.uri, lineRange);
                }
            }
            
            return null;
        }
    });

    // Document Symbol Provider
    const symbolProvider = vscode.languages.registerDocumentSymbolProvider(selector, {
        provideDocumentSymbols(document: vscode.TextDocument) {
            const symbols: vscode.SymbolInformation[] = [];
            const text = document.getText();
            const lines = text.split('\n');
            
            let inFunction = false;
            let functionName = '';
            let functionLine = 0;
             
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i];
                const lineRange = new vscode.Range(i, 0, i, line.length);
                
                // Functions
                const fnMatch = line.match(/(?:pub\s+)?(?:async\s+)?fn\s+(\w+)/);
                if (fnMatch) {
                    symbols.push(new vscode.SymbolInformation(
                        fnMatch[1],
                        vscode.SymbolKind.Function,
                        lineRange
                    ));
                }
                
                // Structs
                const structMatch = line.match(/(?:pub\s+)?struct\s+(\w+)/);
                if (structMatch) {
                    symbols.push(new vscode.SymbolInformation(
                        structMatch[1],
                        vscode.SymbolKind.Struct,
                        lineRange
                    ));
                }
                
                // Enums
                const enumMatch = line.match(/(?:pub\s+)?enum\s+(\w+)/);
                if (enumMatch) {
                    symbols.push(new vscode.SymbolInformation(
                        enumMatch[1],
                        vscode.SymbolKind.Enum,
                        lineRange
                    ));
                }
                
                // Traits
                const traitMatch = line.match(/(?:pub\s+)?trait\s+(\w+)/);
                if (traitMatch) {
                    symbols.push(new vscode.SymbolInformation(
                        traitMatch[1],
                        vscode.SymbolKind.Interface,
                        lineRange
                    ));
                }
                
                // Classes
                const classMatch = line.match(/(?:pub\s+)?class\s+(\w+)/);
                if (classMatch) {
                    symbols.push(new vscode.SymbolInformation(
                        classMatch[1],
                        vscode.SymbolKind.Class,
                        lineRange
                    ));
                }
                
                // Variables
                const letMatch = line.match(/let\s+(\w+)/);
                if (letMatch) {
                    symbols.push(new vscode.SymbolInformation(
                        letMatch[1],
                        vscode.SymbolKind.Variable,
                        lineRange
                    ));
                }
            }
            
            return symbols;
        }
    });

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('lexicon.run', () => {
            vscode.window.showInformationMessage('🤖 Running Lexicon file...');
            const terminal = vscode.window.createTerminal('Lexicon Run');
            terminal.sendText('lex run $EDITOR_FILE');
            terminal.show();
        }),
        vscode.commands.registerCommand('lexicon.newProject', () => {
            vscode.window.showInformationMessage('📁 Creating new Lexicon project...');
        }),
        vscode.commands.registerCommand('lexicon.build', () => {
            vscode.window.showInformationMessage('🔨 Building Lexicon project...');
        }),
        vscode.commands.registerCommand('lexicon.test', () => {
            vscode.window.showInformationMessage('🧪 Running Lexicon tests...');
        })
    );

    context.subscriptions.push(
        completionProvider,
        hoverProvider,
        definitionProvider,
        symbolProvider
    );
}

export function deactivate() {}
