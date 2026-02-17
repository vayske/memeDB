import { useEffect, useState, useRef } from 'react';

interface Image {
  id: number;
  url: string;
  filename: string;
  ext: string;
}

// 新增：标签的数据结构
interface Tag {
  id: number;
  name: string;
}

interface ApiResponse<T> {
  code: number;
  msg: string;
  data: T;
}

function App() {
  const [images, setImages] = useState<Image[]>([]);
  const [loading, setLoading] = useState(false);
  const [searchTags, setSearchTags] = useState('');
  const [isUploading, setIsUploading] = useState(false);
  const [isDragging, setIsDragging] = useState(false);

  // === 新增状态：控制弹窗 ===
  const [selectedImage, setSelectedImage] = useState<Image | null>(null);
  const [tagInput, setTagInput] = useState('');
  const [currentTags, setCurrentTags] = useState<string[]>([]); // 当前选中图片的标签

  const fileInputRef = useRef<HTMLInputElement>(null);

  // 获取图片列表
  const fetchImages = async (tags: string = '') => {
    setLoading(true);
    try {
      const query = tags ? `?tags=${tags}` : '';
      const res = await fetch(`/api/search${query}`);
      const json: ApiResponse<Image[]> = await res.json();
      if (json.code === 200) setImages(json.data);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const uploadFile = async (file: File) => {
    setIsUploading(true);
    const formData = new FormData();
    formData.append('file', file);

    try {
      const res = await fetch('/api/upload', { method: 'POST', body: formData });

      if (!res.ok) {
        const errorText = await res.text();
        alert(`上传失败: ${errorText}`);
        return;
      }

      const json = await res.json();
      if (json.code === 200) {
        // alert('上传成功'); // 可以注释掉这个烦人的弹窗，让用户直接看到图出来
        fetchImages(searchTags);
      } else {
        alert('业务错误: ' + json.msg);
      }
    } catch (err) {
      console.error(err);
      alert('网络错误');
    } finally {
      setIsUploading(false);
      // 清空 input，允许重复上传同名文件
      if (fileInputRef.current) fileInputRef.current.value = '';
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) uploadFile(file);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault(); // 必须阻止默认行为，否则浏览器会直接打开图片
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const file = e.dataTransfer.files?.[0];
    if (file && file.type.startsWith('image/')) {
      uploadFile(file);
    } else {
      alert('请拖入图片文件！');
    }
  };

  // === 新增功能：添加标签 ===
  const handleAddTag = async () => {
    if (!selectedImage || !tagInput.trim()) return;

    // 乐观更新 (UI先反应，不等接口)
    const newTag = tagInput.trim();
    setCurrentTags(prev => [...prev, newTag]);
    setTagInput('');

    try {
      await fetch(`/api/images/${selectedImage.id}/tags`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tags: [newTag] }),
      });
      // 实际场景中，这里应该重新 fetch 标签确认，但为了流畅先这样
    } catch (err) {
      console.error('Failed to add tag', err);
      alert('打标签失败');
    }
  };

  const handleDelete = async () => {
    if (!selectedImage) return;

    // 1. 二次确认 (防止手滑)
    if (!confirm('Are you sure you want to delete this meme? This action cannot be undone.')) {
      return;
    }

    try {
      // 2. 发送 DELETE 请求
      const res = await fetch(`/api/images/${selectedImage.id}`, {
        method: 'DELETE',
      });
      const json = await res.json();

      if (json.code === 200) {
        // 3. 成功后的处理
        alert('Deleted successfully');
        setSelectedImage(null); // 关闭弹窗
        fetchImages(searchTags); // 刷新列表
      } else {
        alert('Delete failed: ' + json.msg);
      }
    } catch (err) {
      console.error(err);
      alert('Network error');
    }
  };

  // === 新增功能：点击图片打开详情 ===
  const openModal = async (img: Image) => {
    setSelectedImage(img);
    setCurrentTags([]); // 重置标签显示
    try {
      // 2. 发起请求
      const res = await fetch(`/api/images/${img.id}/tags`);
      const json = await res.json();

      if (json.code === 200) {
        // 3. 提取标签名 (后端返回的是对象数组 [{id:1, name:"cat"}], 我们只需要 name)
        const tagNames = json.data.map((t: Tag) => t.name);
        setCurrentTags(tagNames);
      }
    } catch (err) {
      console.error("Failed to fetch tags for image", err);
    }
  };

  useEffect(() => { fetchImages(); }, []);

  return (
    <div className="min-h-screen bg-slate-50 text-slate-900 font-sans pb-20">
      {/* Header */}
      <div className="bg-white border-b border-slate-200 sticky top-0 z-10 shadow-sm px-4 h-16 flex items-center justify-between">
        <div className="flex items-center gap-2 font-bold text-xl">
          <span>🐸</span> <span className="text-blue-600">MemeDB</span>
        </div>
        <div className="flex-1 max-w-md mx-4">
          <input
            type="text"
            placeholder="Search tags..."
            className="w-full bg-slate-100 rounded-full px-4 py-2 outline-none focus:ring-2 focus:ring-blue-500"
            value={searchTags}
            onChange={e => setSearchTags(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && fetchImages(searchTags)}
          />
        </div>
        <input type="file" ref={fileInputRef} className="hidden" accept="image/*" onChange={handleInputChange} />
        <button
          disabled={isUploading}
          onClick={() => fileInputRef.current?.click()}
          className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium disabled:opacity-50 transition"
        >
          {isUploading ? 'Uploading...' : 'Upload'}
        </button>
      </div>

      {/* Grid */}
      <main className="max-w-7xl mx-auto p-4">
        {/* === 新增：拖拽上传区域 === */}
        <div
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onClick={() => fileInputRef.current?.click()} // 点击也能上传
          className={`
            mb-8 rounded-xl border-2 border-dashed transition-all duration-200 cursor-pointer
            flex flex-col items-center justify-center py-12 px-4 text-center
            ${isDragging
                    ? 'border-blue-500 bg-blue-50 scale-[1.01] shadow-lg' // 拖拽时的样式
                    : 'border-slate-300 bg-white hover:border-blue-400 hover:bg-slate-50' // 平常的样式
                  }
          `}
        >
          {/* 这里可以用 Lucide-React 的图标，或者简单的 Emoji */}
          <div className="text-4xl mb-4">
            {isUploading ? '⏳' : (isDragging ? '📂' : '☁️')}
          </div>

          {isUploading ? (
            <p className="text-lg font-medium text-slate-500">Uploading meme...</p>
          ) : (
            <>
              <h3 className="text-lg font-bold text-slate-700">
                {isDragging ? 'Drop it like it\'s hot! 🔥' : 'Click or Drag images here'}
              </h3>
              <p className="text-sm text-slate-400 mt-1">
                Supports JPG, PNG, GIF, WEBP
              </p>
            </>
          )}
        </div>
        {loading ? <div className="text-center py-10 text-slate-400">Loading...</div> : (
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {images.map((img) => (
              <div
                key={img.id}
                onClick={() => openModal(img)}
                className="group relative bg-white rounded-xl overflow-hidden shadow-sm hover:shadow-md cursor-pointer border border-slate-100"
              >
                <div className="aspect-square bg-slate-100">
                  <img src={img.url} className="w-full h-full object-cover group-hover:scale-105 transition duration-500" />
                </div>
                <div className="p-2 flex justify-between items-center bg-white">
                  <span className="text-xs text-slate-400 truncate flex-1">{img.filename}</span>
                  <span className="text-[10px] font-bold bg-slate-100 text-slate-500 px-1 rounded ml-2">
                    {img.ext.toUpperCase()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </main>

      {/* === 详情弹窗 (Modal) === */}
      {selectedImage && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm" onClick={() => setSelectedImage(null)}>
          <div className="bg-white rounded-2xl shadow-2xl max-w-4xl w-full max-h-[90vh] overflow-hidden flex flex-col md:flex-row" onClick={e => e.stopPropagation()}>

            {/* 左侧：大图 */}
            <div className="md:w-2/3 bg-black flex items-center justify-center p-4">
              <img src={selectedImage.url} className="max-w-full max-h-[80vh] object-contain" />
            </div>

            {/* 右侧：操作区 */}
            <div className="md:w-1/3 p-6 flex flex-col bg-white">
              <h2 className="text-lg font-bold truncate mb-1" title={selectedImage.filename}>{selectedImage.filename}</h2>
              <div className="text-sm text-slate-500 mb-6 flex gap-2">
                <span>ID: {selectedImage.id}</span>
                <span>•</span>
                <span>{selectedImage.ext.toUpperCase()}</span>
              </div>

              {/* 标签列表展示 */}
              <div className="flex-1 overflow-y-auto mb-4">
                <h3 className="text-sm font-semibold text-slate-700 mb-2">Tags</h3>
                <div className="flex flex-wrap gap-2">
                  {currentTags.length === 0 && <span className="text-sm text-slate-400 italic">No tags added in this session.</span>}
                  {currentTags.map((tag, idx) => (
                    <span key={idx} className="bg-blue-50 text-blue-600 px-2 py-1 rounded text-sm font-medium">
                      #{tag}
                    </span>
                  ))}
                </div>
              </div>
              <div className="mt-auto"> {/* 确保沉底 */}
                {/* A. 添加标签区域 */}
                <div className="flex gap-2 mb-6"> {/* mb-6 拉开与删除按钮的距离 */}
                  <input
                    type="text"
                    className="flex-1 bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-blue-500 transition"
                    placeholder="Add a tag..."
                    value={tagInput}
                    onChange={e => setTagInput(e.target.value)}
                    onKeyDown={e => e.key === 'Enter' && handleAddTag()}
                  />
                  <button
                    onClick={handleAddTag}
                    className="bg-slate-900 text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-slate-800 transition active:scale-95"
                  >
                    Add
                  </button>
                </div>

                {/* B. 删除区域 (Danger Zone) */}
                <div className="pt-4 border-t border-slate-100">
                  <button
                    onClick={handleDelete}
                    className="w-full flex items-center justify-center gap-2 text-red-600 hover:bg-red-50 hover:border-red-200 border border-transparent px-4 py-2.5 rounded-lg text-sm font-medium transition duration-200 group"
                  >
                    {/* 加个简单的垃圾桶 SVG 图标，增加辨识度 */}
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="opacity-70 group-hover:opacity-100">
                      <path d="M3 6h18" /><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                    </svg>
                    Delete Image
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

      )}
    </div>
  );
}

export default App;
